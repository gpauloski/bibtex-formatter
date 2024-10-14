use crate::token::{stringify, Token};
use crate::{Error, Result};
use std::iter::Peekable;

#[derive(Debug, PartialEq)]
pub struct Tag {
    name: String,
    content: String,
}

#[derive(Debug, PartialEq)]
pub struct Entry {
    kind: String,
    key: String,
    tags: Vec<Tag>,
}

pub struct Parser<I>
where
    I: Iterator<Item = Token>,
{
    tokens: Peekable<I>,
}

impl<I: Iterator<Item = Token>> Parser<I> {
    pub fn new(iter: I) -> Self {
        Parser {
            tokens: iter.peekable(),
        }
    }

    fn expect(&mut self, expected: Token) -> Result<()> {
        match self.next() {
            Some(token) if token == expected => Ok(()),
            Some(token) => Err(Error::UnexpectedToken(expected, token)),
            None => Err(Error::EndOfTokenStream),
        }
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    fn next(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn parse_content_delim(&mut self, start: Token, end: Token) -> Result<String> {
        match self.next() {
            Some(token) if token == start => (),
            Some(token) => return Err(Error::UnexpectedToken(start, token)),
            None => return Err(Error::EndOfTokenStream),
        }

        let mut tokens: Vec<Token> = Vec::new();

        loop {
            if let Some(token) = self.next() {
                if token == end {
                    break;
                } else {
                    tokens.push(token);
                }
            } else {
                return Err(Error::EndOfTokenStream);
            }
        }

        Ok(stringify(tokens))
    }

    fn parse_content(&mut self) -> Result<String> {
        match self.peek() {
            Some(Token::BraceLeft) => self.parse_content_delim(Token::BraceLeft, Token::BraceRight),
            Some(Token::Quote) => self.parse_content_delim(Token::Quote, Token::Quote),
            Some(token) => Err(Error::ContentParseError(
                format!(
                    "Expected opening brace or quote at start of tag contents but found {:?}",
                    token
                )
                .to_string(),
            )),
            None => Err(Error::EndOfTokenStream),
        }
    }

    fn parse_entry(&mut self) -> Result<Entry> {
        self.expect(Token::At)?;

        if let Some(Token::Value(kind)) = self.next() {
            let mut tags: Vec<Tag> = Vec::new();

            self.expect(Token::BraceLeft)?;

            if let Some(Token::Value(key)) = self.next() {
                loop {
                    if self.peek() == Some(&Token::BraceRight) {
                        self.next();
                        break;
                    }
                    self.expect(Token::Comma)?;
                    tags.push(self.parse_tag()?);
                }

                Ok(Entry { kind, key, tags })
            } else {
                Err(Error::MissingCiteKey)
            }
        } else {
            Err(Error::MissingEntryType)
        }
    }

    fn parse_tag(&mut self) -> Result<Tag> {
        match self.next() {
            Some(Token::Value(name)) => match self.next() {
                Some(Token::Equals) => Ok(Tag {
                    name,
                    content: self.parse_content()?,
                }),
                Some(token) => Err(Error::UnexpectedToken(Token::Equals, token)),
                None => Err(Error::EndOfTokenStream),
            },
            Some(_) => Err(Error::MissingTagName),
            None => Err(Error::EndOfTokenStream),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Entry>> {
        let mut entries: Vec<Entry> = Vec::new();

        while let Some(token) = self.peek() {
            match token {
                Token::At => entries.push(self.parse_entry()?),
                _ => return Err(Error::UnexpectedToken(Token::At, token.clone())),
            }
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_no_tags() {
        let tokens = vec![
            Token::At,
            Token::Value("misc".to_string()),
            Token::BraceLeft,
            Token::Value("citekey".to_string()),
            Token::BraceRight,
        ];
        let mut parser = Parser::new(tokens.into_iter());
        let result = parser.parse();
        let expected = vec![Entry {
            kind: "misc".to_string(),
            key: "citekey".to_string(),
            tags: Vec::with_capacity(0),
        }];
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_with_tags() {
        let tokens = vec![
            Token::At,
            Token::Value("misc".to_string()),
            Token::BraceLeft,
            Token::Value("citekey".to_string()),
            Token::Comma,
            Token::Value("author".to_string()),
            Token::Equals,
            Token::Quote,
            Token::Value("foo".to_string()),
            Token::Quote,
            Token::Comma,
            Token::Value("title".to_string()),
            Token::Equals,
            Token::BraceLeft,
            Token::Value("the".to_string()),
            Token::Comma,
            Token::Value("bar".to_string()),
            Token::BraceRight,
            Token::BraceRight,
        ];
        let mut parser = Parser::new(tokens.into_iter());
        let result = parser.parse();
        let expected = vec![Entry {
            kind: "misc".to_string(),
            key: "citekey".to_string(),
            tags: vec![
                Tag {
                    name: "author".to_string(),
                    content: "foo".to_string(),
                },
                Tag {
                    name: "title".to_string(),
                    content: "the,bar".to_string(),
                },
            ],
        }];
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_missing_type() {
        let tokens = vec![Token::At, Token::BraceLeft, Token::BraceRight];
        let mut parser = Parser::new(tokens.into_iter());
        let result = parser.parse();
        let expected = Err(Error::MissingEntryType);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_missing_key() {
        let tokens = vec![
            Token::At,
            Token::Value("misc".to_string()),
            Token::BraceLeft,
            Token::BraceRight,
        ];
        let mut parser = Parser::new(tokens.into_iter());
        let result = parser.parse();
        let expected = Err(Error::MissingCiteKey);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_missing_equals() {
        let tokens = vec![
            Token::At,
            Token::Value("misc".to_string()),
            Token::BraceLeft,
            Token::Value("citekey".to_string()),
            Token::Comma,
            Token::Value("author".to_string()),
            Token::BraceRight,
        ];
        let mut parser = Parser::new(tokens.into_iter());
        let result = parser.parse();
        let expected = Err(Error::UnexpectedToken(Token::Equals, Token::BraceRight));
        assert_eq!(result, expected);
    }
}
