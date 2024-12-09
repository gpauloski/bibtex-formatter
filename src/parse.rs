use crate::models::{Entries, Entry, RefEntry, StringEntry, Tag};
use crate::token::{stringify, Position, Special, Token, TokenInfo, Whitespace};
use crate::{Error, Result};
use std::iter::Peekable;

pub struct Parser<I>
where
    I: Iterator<Item = TokenInfo>,
{
    tokens: Peekable<I>,
    position: Position,
    remove_empty_tags: bool,
}

impl<I: Iterator<Item = TokenInfo>> Parser<I> {
    // This doesn't implement the Default trait because we do
    // need at least one argument.
    pub fn default(iter: I) -> Self {
        Parser {
            tokens: iter.peekable(),
            position: Position { line: 0, column: 0 },
            remove_empty_tags: true,
        }
    }

    pub fn new(iter: I, remove_empty_tags: bool) -> Self {
        Parser {
            tokens: iter.peekable(),
            position: Position { line: 0, column: 0 },
            remove_empty_tags,
        }
    }

    fn expect(&mut self, expected: Token) -> Result<()> {
        match self.next_non_whitespace() {
            Some(token_info) if token_info.value == expected => Ok(()),
            Some(token_info) => Err(Error::UnexpectedToken(expected, token_info)),
            None => Err(Error::EndOfTokenStream(self.position)),
        }
    }

    fn peek(&mut self) -> Option<&TokenInfo> {
        self.tokens.peek()
    }

    fn peek_non_whitespace(&mut self) -> Option<TokenInfo> {
        while let Some(token_info) = self.peek() {
            if !token_info.is_whitespace() {
                return Some(token_info.clone());
            }
            self.next();
        }
        None
    }

    fn next(&mut self) -> Option<TokenInfo> {
        if let Some(info) = self.tokens.next() {
            self.position = info.position;
            Some(info)
        } else {
            None
        }
    }

    fn next_non_whitespace(&mut self) -> Option<TokenInfo> {
        if let Some(info) = self
            .tokens
            .find(|token_info| !matches!(token_info.value, Token::Whitespace(_)))
        {
            self.position = info.position;
            Some(info)
        } else {
            None
        }
    }

    fn parse_content_delim(&mut self, start: Token, end: Token) -> Result<String> {
        match self.next_non_whitespace() {
            Some(token_info) if token_info.value == start => (),
            Some(token_info) => return Err(Error::UnexpectedToken(start, token_info)),
            None => return Err(Error::EndOfTokenStream(self.position)),
        }

        let mut nested = 0;
        let mut tokens: Vec<Token> = Vec::new();

        loop {
            if let Some(token) = self.next() {
                if start == end && token.value == end {
                    break;
                } else if token.value == start {
                    nested += 1;
                } else if token.value == end && nested == 0 {
                    break;
                } else if token.value == end {
                    nested -= 1;
                }
                if matches!(token.value, Token::Whitespace(_)) {
                    // Skip adding consecutive whitespace tokens
                    if !matches!(tokens.last(), Some(&Token::Whitespace(_))) {
                        tokens.push(Token::Whitespace(Whitespace::Space));
                    }
                } else {
                    tokens.push(token.value);
                }
            } else {
                return Err(Error::EndOfTokenStream(self.position));
            }
        }

        Ok(stringify(tokens))
    }

    fn parse_tag(&mut self) -> Result<Tag> {
        let token_info = match self.next_non_whitespace() {
            Some(token) => token,
            None => return Err(Error::EndOfTokenStream(self.position)),
        };

        let name = match token_info.value {
            Token::Value(name) => name,
            _ => return Err(Error::MissingTagName(token_info)),
        };

        self.expect(Token::Special(Special::Equals))?;

        let content = self.parse_content()?;

        Ok(Tag::new(name, content))
    }

    fn parse_content(&mut self) -> Result<String> {
        let token_info = match self.peek_non_whitespace() {
            Some(token) => token,
            None => return Err(Error::EndOfTokenStream(self.position)),
        };

        match token_info.value {
            // Content can take four forms:
            //   - String delimited by braces
            //   - String delimited by quotes
            //   - A single number (used for years)
            //   - A single word (used for months)
            // The latter two cases are treated similary: check if the content
            // of the tag is a single word followed by a comma or closing brace.
            Token::Special(Special::BraceLeft) => self.parse_content_delim(
                Token::Special(Special::BraceLeft),
                Token::Special(Special::BraceRight),
            ),
            Token::Special(Special::Quote) => self.parse_content_delim(
                Token::Special(Special::Quote),
                Token::Special(Special::Quote),
            ),
            Token::Value(_) => {
                let maybe_content_info = self.next_non_whitespace().unwrap();
                let next_token_info = match self.peek_non_whitespace() {
                    Some(token) => token,
                    None => return Err(Error::EndOfTokenStream(self.position)),
                };
                if !matches!(
                    next_token_info.value,
                    Token::Special(Special::Comma) | Token::Special(Special::BraceRight),
                ) {
                    return Err(Error::UnexpectedToken(
                        Token::Special(Special::Comma),
                        next_token_info,
                    ));
                }

                match maybe_content_info.value {
                    Token::Value(s) => Ok(s),
                    _ => panic!(),
                }
            }
            _ => Err(Error::MissingContentOpenToken(token_info)),
        }
    }

    fn parse_ref_body(&mut self, kind: String) -> Result<RefEntry> {
        let key = match self.next_non_whitespace() {
            Some(token) => match token.value {
                Token::Value(key) => key,
                _ => return Err(Error::MissingCiteKey(token)),
            },
            None => return Err(Error::EndOfTokenStream(self.position)),
        };

        let mut tags: Vec<Tag> = Vec::new();
        loop {
            match self.peek_non_whitespace() {
                Some(token) if token.value == Token::Special(Special::BraceRight) => {
                    self.next_non_whitespace();
                    break;
                }
                Some(token) if token.value == Token::Special(Special::Comma) => {
                    self.next_non_whitespace();
                }
                _ => {
                    tags.push(self.parse_tag()?);
                }
            };
        }

        if self.remove_empty_tags {
            tags.retain(|t| !t.content.trim().is_empty());
        }

        Ok(RefEntry::new(kind, key, tags))
    }

    fn parse_string_body(&mut self) -> Result<StringEntry> {
        let tag = self.parse_tag()?;

        // Ignore optional trailing comma and check for closing brace.
        match self.next_non_whitespace() {
            Some(token) if token.value == Token::Special(Special::BraceRight) => (),
            Some(token) if token.value == Token::Special(Special::Comma) => {
                self.expect(Token::Special(Special::BraceRight))?;
            }
            Some(token) => {
                return Err(Error::UnexpectedToken(
                    Token::Special(Special::BraceRight),
                    token,
                ));
            }
            None => return Err(Error::EndOfTokenStream(self.position)),
        }

        Ok(StringEntry::new(tag.name, tag.content))
    }

    fn parse_entry(&mut self) -> Result<Entry> {
        self.expect(Token::Special(Special::At))?;

        let token_info = match self.next_non_whitespace() {
            Some(token) => token,
            None => return Err(Error::EndOfTokenStream(self.position)),
        };

        let kind = match token_info.value {
            Token::Value(kind) => kind.to_lowercase(),
            _ => return Err(Error::MissingEntryType(token_info)),
        };

        self.expect(Token::Special(Special::BraceLeft))?;

        let entry = match kind.as_str() {
            "string" => Entry::StringEntry(self.parse_string_body()?),
            _ => Entry::RefEntry(self.parse_ref_body(kind)?),
        };

        Ok(entry)
    }

    pub fn parse(&mut self) -> Result<Entries> {
        let mut references: Vec<RefEntry> = Vec::new();
        let mut strings: Vec<StringEntry> = Vec::new();

        while let Some(token_info) = self.peek_non_whitespace() {
            let entry = match token_info.value {
                Token::Special(Special::At) => self.parse_entry()?,
                _ => {
                    return Err(Error::UnexpectedToken(
                        Token::Special(Special::At),
                        // Should never be None because peek_non_whitespace()
                        // returned Some(_).
                        self.next().unwrap(),
                    ));
                }
            };

            match entry {
                Entry::RefEntry(e) => references.push(e),
                Entry::StringEntry(e) => strings.push(e),
            };
        }

        Ok(Entries {
            references,
            strings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_no_tags() {
        let tokens = vec![
            Token::Special(Special::At),
            Token::Value("misc".to_string()),
            Token::Special(Special::BraceLeft),
            Token::Value("citekey".to_string()),
            Token::Special(Special::BraceRight),
        ];
        let iter = tokens.into_iter().map(|token| TokenInfo {
            value: token,
            position: Position { line: 0, column: 0 },
        });
        let mut parser = Parser::default(iter);
        let result = parser.parse().unwrap();
        let expected = vec![RefEntry {
            kind: "misc".to_string(),
            key: "citekey".to_string(),
            tags: Vec::with_capacity(0),
        }];
        assert_eq!(result.references, expected);
    }

    #[test]
    fn test_parse_with_tags() {
        let tokens = vec![
            Token::Special(Special::At),
            Token::Value("misc".to_string()),
            Token::Special(Special::BraceLeft),
            Token::Value("citekey".to_string()),
            Token::Special(Special::Comma),
            Token::Value("author".to_string()),
            Token::Special(Special::Equals),
            Token::Special(Special::Quote),
            Token::Value("foo".to_string()),
            Token::Special(Special::Quote),
            Token::Special(Special::Comma),
            Token::Value("title".to_string()),
            Token::Special(Special::Equals),
            Token::Special(Special::BraceLeft),
            Token::Value("the".to_string()),
            Token::Special(Special::Comma),
            Token::Value("bar".to_string()),
            Token::Special(Special::BraceRight),
            Token::Special(Special::BraceRight),
        ];
        let iter = tokens.into_iter().map(|token| TokenInfo {
            value: token,
            position: Position { line: 0, column: 0 },
        });
        let mut parser = Parser::default(iter);
        let result = parser.parse().unwrap();
        let expected = vec![RefEntry {
            kind: "misc".to_string(),
            key: "citekey".to_string(),
            tags: vec![
                Tag {
                    name: "title".to_string(),
                    content: "the,bar".to_string(),
                },
                Tag {
                    name: "author".to_string(),
                    content: "foo".to_string(),
                },
            ],
        }];
        assert_eq!(result.references, expected);
    }

    #[test]
    fn test_parse_missing_type() {
        let tokens = vec![
            Token::Special(Special::At),
            Token::Special(Special::BraceLeft),
            Token::Special(Special::BraceRight),
        ];
        let iter = tokens.into_iter().map(|token| TokenInfo {
            value: token,
            position: Position { line: 0, column: 0 },
        });
        let mut parser = Parser::default(iter);
        let result = parser.parse().unwrap_err();
        assert!(matches!(result, Error::MissingEntryType(_)));
    }

    #[test]
    fn test_parse_missing_key() {
        let tokens = vec![
            Token::Special(Special::At),
            Token::Value("misc".to_string()),
            Token::Special(Special::BraceLeft),
            Token::Special(Special::BraceRight),
        ];
        let iter = tokens.into_iter().map(|token| TokenInfo {
            value: token,
            position: Position { line: 0, column: 0 },
        });
        let mut parser = Parser::default(iter);
        let result = parser.parse().unwrap_err();
        assert!(matches!(result, Error::MissingCiteKey(_)));
    }

    #[test]
    fn test_parse_missing_equals() {
        let tokens = vec![
            Token::Special(Special::At),
            Token::Value("misc".to_string()),
            Token::Special(Special::BraceLeft),
            Token::Value("citekey".to_string()),
            Token::Special(Special::Comma),
            Token::Value("author".to_string()),
            Token::Special(Special::BraceRight),
        ];
        let iter = tokens.into_iter().map(|token| TokenInfo {
            value: token,
            position: Position { line: 0, column: 0 },
        });
        let mut parser = Parser::default(iter);
        let result = parser.parse().unwrap_err();
        assert!(matches!(
            result,
            Error::UnexpectedToken(
                Token::Special(Special::Equals),
                TokenInfo {
                    value: Token::Special(Special::BraceRight),
                    position: Position { line: 0, column: 0 },
                },
            )
        ));
    }

    #[test]
    fn test_nested_content() {
        let tokens = vec![
            Token::Special(Special::BraceLeft),
            Token::Special(Special::BraceLeft),
            Token::Value("value".to_string()),
            Token::Special(Special::BraceRight),
            Token::Special(Special::BraceRight),
        ];
        let iter = tokens.into_iter().map(|token| TokenInfo {
            value: token,
            position: Position { line: 0, column: 0 },
        });
        let mut parser = Parser::default(iter);
        let result = parser.parse_content().unwrap();
        assert_eq!(result, "{value}");
    }

    #[test]
    fn test_remove_empty_tags() {
        let tokens = vec![
            Token::Special(Special::At),
            Token::Value("misc".to_string()),
            Token::Special(Special::BraceLeft),
            Token::Value("citekey".to_string()),
            Token::Special(Special::Comma),
            Token::Value("author".to_string()),
            Token::Special(Special::Equals),
            Token::Special(Special::Quote),
            Token::Special(Special::Quote),
            Token::Special(Special::BraceRight),
        ];
        let iter = tokens.into_iter().map(|token| TokenInfo {
            value: token,
            position: Position { line: 0, column: 0 },
        });
        let mut parser = Parser::default(iter);
        let result = parser.parse().unwrap();
        let expected = vec![RefEntry {
            kind: "misc".to_string(),
            key: "citekey".to_string(),
            tags: vec![],
        }];
        assert_eq!(result.references, expected);
    }

    #[test]
    fn test_retain_empty_tags() {
        let tokens = vec![
            Token::Special(Special::At),
            Token::Value("misc".to_string()),
            Token::Special(Special::BraceLeft),
            Token::Value("citekey".to_string()),
            Token::Special(Special::Comma),
            Token::Value("author".to_string()),
            Token::Special(Special::Equals),
            Token::Special(Special::Quote),
            Token::Special(Special::Quote),
            Token::Special(Special::BraceRight),
        ];
        let iter = tokens.into_iter().map(|token| TokenInfo {
            value: token,
            position: Position { line: 0, column: 0 },
        });
        let mut parser = Parser::new(iter, false);
        let result = parser.parse().unwrap();
        let expected = vec![RefEntry {
            kind: "misc".to_string(),
            key: "citekey".to_string(),
            tags: vec![Tag {
                name: "author".to_string(),
                content: "".to_string(),
            }],
        }];
        assert_eq!(result.references, expected);
    }

    #[test]
    fn test_parse_strings() {
        let tokens = vec![
            Token::Special(Special::At),
            Token::Value("string".to_string()),
            Token::Special(Special::BraceLeft),
            Token::Value("acm".to_string()),
            Token::Special(Special::Equals),
            Token::Special(Special::Quote),
            Token::Value("Association for Computing Machinery".to_string()),
            Token::Special(Special::Quote),
            Token::Special(Special::BraceRight),
            Token::Whitespace(Whitespace::NewLine),
            Token::Special(Special::At),
            Token::Value("STRING".to_string()),
            Token::Special(Special::BraceLeft),
            Token::Value("ieee".to_string()),
            Token::Special(Special::Equals),
            Token::Special(Special::Quote),
            Token::Value("Institute of Electrical and Electronics Engineers".to_string()),
            Token::Special(Special::Quote),
            Token::Special(Special::BraceRight),
        ];
        let iter = tokens.into_iter().map(|token| TokenInfo {
            value: token,
            position: Position { line: 0, column: 0 },
        });
        let mut parser = Parser::new(iter, false);
        let result = parser.parse().unwrap();
        let expected = vec![
            StringEntry {
                name: "acm".to_string(),
                content: "Association for Computing Machinery".to_string(),
            },
            StringEntry {
                name: "ieee".to_string(),
                content: "Institute of Electrical and Electronics Engineers".to_string(),
            },
        ];
        assert_eq!(result.strings, expected);
    }
}
