use crate::token::{stringify, Token};
use crate::Result;
use std::iter::Peekable;

struct Tag {
    key: String,
    value: String,
}

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
    index: usize,
}

impl<I: Iterator<Item = Token>> Parser<I> {
    pub fn new(iter: I) -> Self {
        Parser {
            tokens: iter.peekable(),
            index: 0,
        }
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    fn next(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn parse_value_brace(&mut self) -> Result<String> {
        if !(self.next() == Some(Token::BraceLeft)) {
            return Err("Expected { token.".into());
        }

        let mut tokens: Vec<Token> = Vec::new();

        loop {
            if let Some(token) = self.next() {
                match token {
                    Token::BraceRight => break,
                    _ => tokens.push(token),
                }
            } else {
                return Err("Reached end of token stream.".into());
            }
        }

        Ok(stringify(tokens))
    }

    fn parse_value_quote(&mut self) -> Result<String> {
        Err("Unimplemented.".into())
    }

    fn parse_value(&mut self) -> Result<String> {
        match self.peek() {
            Some(Token::BraceLeft) => self.parse_value_brace(),
            Some(Token::Quote) => self.parse_value_quote(),
            _ => Err("Value should start with an opening brace or quote.".into()),
        }
    }

    pub fn parse_entry(&mut self) -> Result<Entry> {
        if !(self.next() == Some(Token::At)) {
            return Err("Entry must start with @.".into());
        }

        if let Some(Token::Value(kind)) = self.next() {
            let mut tags: Vec<Tag> = Vec::new();

            if !(self.next() == Some(Token::BraceLeft)) {
                return Err("Opening brace expected after entry type.".into());
            }

            if let Some(Token::Value(key)) = self.next() {
                loop {
                    tags.push(self.parse_tag()?);
                    if self.peek() == Some(&Token::BraceRight) {
                        self.next();
                        break;
                    } else if let Some(Token::Comma) = self.next() {
                        // Expected
                    } else {
                        return Err("Expected comma after entry tag.".into());
                    }
                }

                Ok(Entry { kind, key, tags })
            } else {
                Err("Missing key in entry.".into())
            }
        } else {
            Err("Missing entry type after @.".into())
        }
    }

    pub fn parse_tag(&mut self) -> Result<Tag> {
        if let Some(Token::Value(key)) = self.next() {
            if !(self.next() == Some(Token::Equals)) {
                return Err("Expected equals after tag key.".into());
            }
            Ok(Tag {
                key,
                value: self.parse_value()?,
            })
        } else {
            Err("Tag is missing key.".into())
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Entry>> {
        let mut entries: Vec<Entry> = Vec::new();

        while let Some(token) = self.peek() {
            match token {
                Token::At => entries.push(self.parse_entry()?),
                _ => return Err("Undefined.".into()),
            }
        }

        Ok(entries)
    }
}
