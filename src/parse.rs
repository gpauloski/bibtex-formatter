use crate::models::{CommentEntry, Entries, EntryType, PreambleEntry, RefEntry, StringEntry};
use crate::models::{Part, Sequence, Tag, Value};
use crate::token::{stringify, Position, Special, Token, TokenInfo, Whitespace};
use crate::{Error, Result};
use std::iter::Peekable;

pub struct Parser<I>
where
    I: Iterator<Item = TokenInfo>,
{
    tokens: Peekable<I>,
    position: Position,
}

impl<I: Iterator<Item = TokenInfo>> Parser<I> {
    pub fn new(iter: I) -> Self {
        Self {
            tokens: iter.peekable(),
            position: Position { line: 0, column: 0 },
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

    pub fn parse(&mut self) -> Result<Entries> {
        let mut entries: Vec<EntryType> = Vec::new();

        while let Some(token_info) = self.peek_non_whitespace() {
            let entry = match token_info.value {
                Token::Special(Special::At) => self.parse_entry()?,
                _ => {
                    if let Some(token_info) = self.next() {
                        return Err(Error::UnexpectedToken(
                            Token::Special(Special::At),
                            token_info,
                        ));
                    } else {
                        return Err(Error::InternalAssertion(
                            "Peeked token return none.".to_string(),
                        ));
                    };
                }
            };
            entries.push(entry);
        }

        Ok(Entries::new(entries))
    }

    fn parse_entry(&mut self) -> Result<EntryType> {
        self.expect(Token::Special(Special::At))?;

        let token_info = match self.next_non_whitespace() {
            Some(token) => token,
            None => return Err(Error::EndOfTokenStream(self.position)),
        };

        let kind = match token_info.value {
            Token::Value(kind) => kind,
            _ => return Err(Error::MissingEntryType(token_info)),
        };

        let entry = match kind.to_lowercase().as_str() {
            "comment" => EntryType::CommentEntry(self.parse_comment_entry()?),
            "preamble" => EntryType::PreambleEntry(self.parse_preamble_entry()?),
            "string" => EntryType::StringEntry(self.parse_string_entry()?),
            _ => EntryType::RefEntry(self.parse_ref_entry(kind)?),
        };

        Ok(entry)
    }

    fn parse_comment_entry(&mut self) -> Result<CommentEntry> {
        self.expect(Token::Special(Special::BraceLeft))?;

        let mut tokens: Vec<Token> = Vec::new();

        loop {
            if let Some(token_info) = self.next() {
                match token_info.value {
                    Token::Special(Special::BraceRight) => break,
                    value => tokens.push(value),
                }
            } else {
                return Err(Error::EndOfTokenStream(self.position));
            }
        }

        Ok(CommentEntry::new(stringify(tokens)))
    }

    fn parse_preamble_entry(&mut self) -> Result<PreambleEntry> {
        self.expect(Token::Special(Special::BraceLeft))?;
        let seq = self.parse_tag_value_sequence()?;
        self.expect(Token::Special(Special::BraceRight))?;
        Ok(PreambleEntry::new(seq))
    }

    fn parse_string_entry(&mut self) -> Result<StringEntry> {
        self.expect(Token::Special(Special::BraceLeft))?;

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

        Ok(StringEntry::new(tag))
    }

    fn parse_ref_entry(&mut self, kind: String) -> Result<RefEntry> {
        self.expect(Token::Special(Special::BraceLeft))?;

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

        Ok(RefEntry::new(kind, key, tags))
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

        let value = self.parse_tag_value()?;

        Ok(Tag::new(name, value))
    }

    fn parse_tag_value(&mut self) -> Result<Value> {
        let token_info = match self.peek_non_whitespace() {
            Some(token) => token,
            None => return Err(Error::EndOfTokenStream(self.position)),
        };

        match token_info.value {
            // A tag's value can take three forms (as defined by the Value enum):
            //   - A single string delimited by braces
            //   - A single integer
            //   - A sequence of one or more parts joined by pound (#) signs
            //     where parts string variable names or a quote delimted string
            // In practice, we treat the last two as the same and see if the
            // resulting sequence is length one containing and integer to
            // resolve the difference.
            Token::Special(Special::BraceLeft) => {
                let s = self.parse_delimited_string(
                    Token::Special(Special::BraceLeft),
                    Token::Special(Special::BraceRight),
                )?;
                Ok(Value::Single(s.trim().to_string()))
            }
            Token::Special(Special::Quote) | Token::Value(_) => {
                let mut seq = self.parse_tag_value_sequence()?;
                if seq.len() == 1 {
                    match seq.next() {
                        Some(Part::Quoted(s)) => Ok(Value::Single(s.trim().to_string())),
                        Some(Part::Value(s)) => {
                            let value = s.parse::<u64>().map_or_else(
                                |_| Value::Sequence(Sequence::new(vec![Part::Value(s)])),
                                Value::Integer,
                            );
                            Ok(value)
                        }
                        None => Err(Error::InternalAssertion(
                            "Failed to get item from sequence of length one.".to_string(),
                        )),
                    }
                } else {
                    Ok(Value::Sequence(seq))
                }
            }
            _ => Err(Error::MissingContent(token_info)),
        }
    }

    fn parse_tag_value_sequence(&mut self) -> Result<Sequence> {
        let mut parts: Vec<Part> = Vec::new();

        // Get the first word/string since there must be at least one.
        parts.push(self.parse_tag_value_part()?);

        loop {
            if let Some(token_info) = self.peek_non_whitespace() {
                match token_info.value {
                    Token::Special(Special::BraceRight) => break,
                    Token::Special(Special::Comma) => break,
                    Token::Special(Special::Pound) => {
                        self.expect(Token::Special(Special::Pound))?;
                        parts.push(self.parse_tag_value_part()?);
                    }
                    _ => {
                        return Err(Error::UnexpectedToken(
                            Token::Special(Special::Comma),
                            token_info,
                        ))
                    }
                };
            } else {
                return Err(Error::EndOfTokenStream(self.position));
            }
        }

        Ok(Sequence::new(parts))
    }

    fn parse_tag_value_part(&mut self) -> Result<Part> {
        if let Some(token_info) = self.peek_non_whitespace() {
            match token_info.value {
                Token::Special(Special::Quote) => {
                    let s = self.parse_delimited_string(
                        Token::Special(Special::Quote),
                        Token::Special(Special::Quote),
                    )?;
                    Ok(Part::Quoted(s))
                }
                Token::Value(_) => {
                    if let Some(token_info) = self.next_non_whitespace() {
                        if let Token::Value(s) = token_info.value {
                            Ok(Part::Value(s))
                        } else {
                            Err(Error::InternalAssertion(
                                "Token should be value type.".to_string(),
                            ))
                        }
                    } else {
                        Err(Error::InternalAssertion(
                            "Peeked token return none.".to_string(),
                        ))
                    }
                }
                _ => Err(Error::MissingContent(token_info)),
            }
        } else {
            Err(Error::EndOfTokenStream(self.position))
        }
    }

    fn parse_delimited_string(&mut self, start: Token, end: Token) -> Result<String> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn as_iter(tokens: Vec<Token>) -> impl Iterator<Item = TokenInfo> {
        tokens.into_iter().enumerate().map(|(i, token)| TokenInfo {
            value: token,
            position: Position {
                line: i as u32,
                column: 0,
            },
        })
    }

    #[test]
    fn test_parse_comment_entry() -> Result<()> {
        let tokens = vec![
            Token::Special(Special::At),
            Token::Value("comment".to_string()),
            Token::Special(Special::BraceLeft),
            Token::Whitespace(Whitespace::Space),
            Token::Value("value".to_string()),
            Token::Whitespace(Whitespace::Space),
            Token::Special(Special::BraceRight),
        ];
        let mut parser = Parser::new(as_iter(tokens));

        let entry = parser.parse_entry()?;
        let expected = EntryType::CommentEntry(CommentEntry::new(" value ".to_string()));
        assert_eq!(entry, expected);

        Ok(())
    }

    #[test]
    fn test_parse_preamble_entries() -> Result<()> {
        let tokens = vec![
            Token::Special(Special::At),
            Token::Value("preamble".to_string()),
            Token::Special(Special::BraceLeft),
            Token::Special(Special::Quote),
            Token::Value("test".to_string()),
            Token::Whitespace(Whitespace::Space),
            Token::Value("string".to_string()),
            Token::Special(Special::Quote),
            Token::Special(Special::Pound),
            Token::Value("value".to_string()),
            Token::Special(Special::BraceRight),
        ];
        let mut parser = Parser::new(as_iter(tokens));

        let entry = parser.parse_entry()?;
        let seq = Sequence::new(vec![
            Part::Quoted("test string".to_string()),
            Part::Value("value".to_string()),
        ]);
        let expected = EntryType::PreambleEntry(PreambleEntry::new(seq));
        assert_eq!(entry, expected);

        Ok(())
    }

    #[test]
    fn test_parse_string_entries() -> Result<()> {
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
        let mut parser = Parser::new(as_iter(tokens));

        let entries = parser.parse()?;
        let expected = Entries::new(vec![
            EntryType::StringEntry(StringEntry::new(Tag::new(
                "acm".to_string(),
                Value::Single("Association for Computing Machinery".to_string()),
            ))),
            EntryType::StringEntry(StringEntry::new(Tag::new(
                "ieee".to_string(),
                Value::Single("Institute of Electrical and Electronics Engineers".to_string()),
            ))),
        ]);
        assert_eq!(entries, expected);

        Ok(())
    }

    #[test]
    fn test_parse_ref_entry() -> Result<()> {
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
        let mut parser = Parser::new(as_iter(tokens));

        let entry = parser.parse_entry()?;
        let tags = vec![
            Tag::new("author".to_string(), Value::Single("foo".to_string())),
            Tag::new("title".to_string(), Value::Single("the,bar".to_string())),
        ];
        let expected = EntryType::RefEntry(RefEntry::new(
            "misc".to_string(),
            "citekey".to_string(),
            tags,
        ));
        assert_eq!(entry, expected);

        Ok(())
    }

    #[test]
    fn test_parse_ref_entry_no_tags() -> Result<()> {
        let tokens = vec![
            Token::Special(Special::At),
            Token::Value("misc".to_string()),
            Token::Special(Special::BraceLeft),
            Token::Value("citekey".to_string()),
            Token::Special(Special::BraceRight),
        ];
        let mut parser = Parser::new(as_iter(tokens));

        let entry = parser.parse_entry()?;
        let expected = EntryType::RefEntry(RefEntry::new(
            "misc".to_string(),
            "citekey".to_string(),
            Vec::with_capacity(0),
        ));
        assert_eq!(entry, expected);

        Ok(())
    }

    #[test]
    fn test_parse_ref_entry_retain_empty_tags() -> Result<()> {
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
        let mut parser = Parser::new(as_iter(tokens));

        let entry = parser.parse_entry()?;
        let expected = EntryType::RefEntry(RefEntry::new(
            "misc".to_string(),
            "citekey".to_string(),
            vec![Tag::new(
                "author".to_string(),
                Value::Single("".to_string()),
            )],
        ));
        assert_eq!(entry, expected);

        Ok(())
    }

    #[test]
    fn test_parse_tag() -> Result<()> {
        let tokens = vec![
            Token::Value("name".to_string()),
            Token::Special(Special::Equals),
            Token::Special(Special::BraceLeft),
            Token::Value("test".to_string()),
            Token::Whitespace(Whitespace::Space),
            Token::Value("string".to_string()),
            Token::Special(Special::BraceRight),
        ];
        let mut parser = Parser::new(as_iter(tokens));

        let parsed = parser.parse_tag()?;
        let expected = Tag::new("name".to_string(), Value::Single("test string".to_string()));
        assert_eq!(parsed, expected);

        Ok(())
    }

    #[test]
    fn test_parse_tag_value_single_braces() -> Result<()> {
        let tokens = vec![
            Token::Special(Special::BraceLeft),
            Token::Value("test".to_string()),
            Token::Whitespace(Whitespace::Space),
            Token::Value("string".to_string()),
            Token::Special(Special::BraceRight),
        ];
        let mut parser = Parser::new(as_iter(tokens));

        let parsed = parser.parse_tag_value()?;
        let expected = Value::Single("test string".to_string());
        assert_eq!(parsed, expected);

        Ok(())
    }

    #[test]
    fn test_parse_tag_value_single_quotes() -> Result<()> {
        let tokens = vec![
            Token::Special(Special::Quote),
            Token::Value("test".to_string()),
            Token::Whitespace(Whitespace::Space),
            Token::Value("string".to_string()),
            Token::Special(Special::Quote),
            Token::Special(Special::Comma),
        ];
        let mut parser = Parser::new(as_iter(tokens));

        let parsed = parser.parse_tag_value()?;
        let expected = Value::Single("test string".to_string());
        assert_eq!(parsed, expected);

        Ok(())
    }

    #[test]
    fn test_parse_tag_value_integer() -> Result<()> {
        let tokens = vec![
            Token::Value("42".to_string()),
            Token::Special(Special::Comma),
        ];
        let mut parser = Parser::new(as_iter(tokens));

        let parsed = parser.parse_tag_value()?;
        let expected = Value::Integer(42);
        assert_eq!(parsed, expected);

        Ok(())
    }

    #[test]
    fn test_parse_tag_value_sequence() -> Result<()> {
        let tokens = vec![
            Token::Special(Special::Quote),
            Token::Value("test".to_string()),
            Token::Whitespace(Whitespace::Space),
            Token::Value("string".to_string()),
            Token::Special(Special::Quote),
            Token::Special(Special::Pound),
            Token::Value("value".to_string()),
            Token::Special(Special::Comma),
        ];
        let mut parser = Parser::new(as_iter(tokens));

        let parsed = parser.parse_tag_value()?;
        let expected = Value::Sequence(Sequence::new(vec![
            Part::Quoted("test string".to_string()),
            Part::Value("value".to_string()),
        ]));
        assert_eq!(parsed, expected);

        Ok(())
    }

    #[test]
    fn test_parse_tag_value_parts() -> Result<()> {
        let tokens = vec![
            Token::Special(Special::Quote),
            Token::Value("test".to_string()),
            Token::Whitespace(Whitespace::Space),
            Token::Value("string".to_string()),
            Token::Special(Special::Quote),
            Token::Special(Special::Pound),
            Token::Value("value".to_string()),
            Token::Special(Special::Comma),
        ];
        let mut parser = Parser::new(as_iter(tokens));

        let parsed = parser.parse_tag_value_sequence()?;
        let expected = Sequence::new(vec![
            Part::Quoted("test string".to_string()),
            Part::Value("value".to_string()),
        ]);
        assert_eq!(parsed, expected);

        Ok(())
    }

    #[test]
    fn test_parse_tag_value_part() -> Result<()> {
        let tokens = vec![
            Token::Special(Special::Quote),
            Token::Value("test".to_string()),
            Token::Whitespace(Whitespace::Space),
            Token::Value("string".to_string()),
            Token::Special(Special::Quote),
            Token::Value("value".to_string()),
        ];
        let mut parser = Parser::new(as_iter(tokens));

        let part = parser.parse_tag_value_part()?;
        assert_eq!(part, Part::Quoted("test string".to_string()));

        let part = parser.parse_tag_value_part()?;
        assert_eq!(part, Part::Value("value".to_string()));

        Ok(())
    }

    #[test]
    fn test_parse_missing_type() -> Result<()> {
        let tokens = vec![
            Token::Special(Special::At),
            Token::Special(Special::BraceLeft),
            Token::Special(Special::BraceRight),
        ];
        let mut parser = Parser::new(as_iter(tokens));

        let result = parser.parse();
        assert!(matches!(result, Err(Error::MissingEntryType(_))));

        Ok(())
    }

    #[test]
    fn test_parse_missing_key() -> Result<()> {
        let tokens = vec![
            Token::Special(Special::At),
            Token::Value("misc".to_string()),
            Token::Special(Special::BraceLeft),
            Token::Special(Special::BraceRight),
        ];
        let mut parser = Parser::new(as_iter(tokens));

        let result = parser.parse();
        assert!(matches!(result, Err(Error::MissingCiteKey(_))));

        Ok(())
    }

    #[test]
    fn test_parse_missing_equals() -> Result<()> {
        let tokens = vec![
            Token::Special(Special::At),
            Token::Value("misc".to_string()),
            Token::Special(Special::BraceLeft),
            Token::Value("citekey".to_string()),
            Token::Special(Special::Comma),
            Token::Value("author".to_string()),
            Token::Special(Special::BraceRight),
        ];
        let mut parser = Parser::new(as_iter(tokens));

        let result = parser.parse();
        assert!(matches!(
            result,
            Err(Error::UnexpectedToken(
                Token::Special(Special::Equals),
                TokenInfo {
                    value: Token::Special(Special::BraceRight),
                    position: Position { line: 6, column: 0 },
                },
            ))
        ));

        Ok(())
    }

    #[test]
    fn test_parse_delimited_string() -> Result<()> {
        let tokens = vec![
            Token::Special(Special::BraceLeft),
            Token::Value("test".to_string()),
            Token::Whitespace(Whitespace::Space),
            Token::Value("string".to_string()),
            Token::Special(Special::BraceRight),
        ];
        let mut parser = Parser::new(as_iter(tokens));

        let parsed = parser.parse_delimited_string(
            Token::Special(Special::BraceLeft),
            Token::Special(Special::BraceRight),
        )?;
        assert_eq!(parsed, "test string");

        Ok(())
    }

    #[test]
    fn test_parse_delimited_string_nested_braces() -> Result<()> {
        let tokens = vec![
            Token::Special(Special::BraceLeft),
            Token::Special(Special::BraceLeft),
            Token::Value("value".to_string()),
            Token::Special(Special::BraceRight),
            Token::Special(Special::BraceRight),
        ];
        let mut parser = Parser::new(as_iter(tokens));

        let parsed = parser.parse_delimited_string(
            Token::Special(Special::BraceLeft),
            Token::Special(Special::BraceRight),
        )?;
        assert_eq!(parsed, "{value}");

        Ok(())
    }
}
