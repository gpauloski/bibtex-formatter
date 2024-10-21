use std::iter::Peekable;

#[derive(Clone, Debug, PartialEq)]
pub enum Special {
    At,
    BraceLeft,
    BraceRight,
    Comma,
    Equals,
    Quote,
}

impl Special {
    pub fn from(c: &char) -> Option<Self> {
        match c {
            '@' => Some(Special::At),
            '{' => Some(Special::BraceLeft),
            '}' => Some(Special::BraceRight),
            ',' => Some(Special::Comma),
            '=' => Some(Special::Equals),
            '"' => Some(Special::Quote),
            _ => None,
        }
    }

    pub fn is_special(c: &char) -> bool {
        matches!(c, '@' | '{' | '}' | ',' | '=' | '"')
    }

    pub fn as_char(&self) -> char {
        match self {
            Self::At => '@',
            Self::BraceLeft => '{',
            Self::BraceRight => '}',
            Self::Comma => ',',
            Self::Equals => '=',
            Self::Quote => '"',
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Whitespace {
    NewLine,
    Space,
    Tab,
}

impl Whitespace {
    pub fn from(c: &char) -> Option<Self> {
        match c {
            '\n' | '\r' => Some(Whitespace::NewLine),
            '\t' => Some(Whitespace::Tab),
            c if c.is_whitespace() => Some(Whitespace::Space),
            _ => None,
        }
    }

    pub fn as_char(&self) -> char {
        match self {
            Self::NewLine => '\n',
            Self::Space => ' ',
            Self::Tab => '\t',
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Special(Special),
    Value(String),
    Whitespace(Whitespace),
}

#[derive(Clone, Debug)]
pub struct TokenInfo {
    pub value: Token,
    pub line: u32,
    pub column: u32,
}

impl TokenInfo {
    pub fn is_special(&self) -> bool {
        matches!(self.value, Token::Special(_))
    }

    pub fn is_value(&self) -> bool {
        matches!(self.value, Token::Value(_))
    }

    pub fn is_whitespace(&self) -> bool {
        matches!(self.value, Token::Whitespace(_))
    }
}

pub struct Tokenizer<I>
where
    I: Iterator<Item = char>,
{
    stream: Peekable<I>,
    line: u32,
    column: u32,
}

impl<I: Iterator<Item = char>> Tokenizer<I> {
    pub fn new(iter: I) -> Self {
        Tokenizer {
            stream: iter.peekable(),
            line: 1,
            column: 0,
        }
    }

    pub fn peek(&mut self) -> Option<&char> {
        self.stream.peek()
    }

    pub fn next(&mut self) -> Option<char> {
        if let Some(next_char) = self.stream.next() {
            if matches!(next_char, '\n' | '\r') {
                self.line += 1;
                self.column = 0;
            }
            self.column += 1;
            Some(next_char)
        } else {
            None
        }
    }

    pub fn tokenize(&mut self) -> Vec<TokenInfo> {
        let mut tokens: Vec<TokenInfo> = Vec::new();

        while let Some(c) = self.next() {
            let token = if let Some(token_type) = Special::from(&c) {
                TokenInfo {
                    value: Token::Special(token_type),
                    line: self.line,
                    column: self.column,
                }
            } else if let Some(token_type) = Whitespace::from(&c) {
                TokenInfo {
                    value: Token::Whitespace(token_type),
                    line: self.line,
                    column: self.column,
                }
            } else {
                let line = self.line;
                let column = self.column;
                let mut value = String::new();

                value.push(c);
                while let Some(c) = self.peek() {
                    if Special::is_special(c) || c.is_whitespace() {
                        break;
                    } else {
                        value.push(self.next().unwrap())
                    }
                }

                TokenInfo {
                    value: Token::Value(value),
                    line,
                    column,
                }
            };
            tokens.push(token);
        }

        tokens
    }
}

pub fn stringify(tokens: Vec<Token>) -> String {
    let capacity = tokens
        .iter()
        .map(|token| match token {
            Token::Value(s) => s.len(),
            _ => 1,
        })
        .sum();

    let mut string = String::with_capacity(capacity);

    for token in tokens {
        match token {
            Token::Special(c) => string.push(c.as_char()),
            Token::Value(s) => string.push_str(&s),
            Token::Whitespace(c) => string.push(c.as_char()),
        };
    }

    string
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let text = "";
        let expected: Vec<Token> = Vec::with_capacity(0);
        let mut tokenizer = Tokenizer::new(text.chars());
        let tokens: Vec<Token> = tokenizer.tokenize().into_iter().map(|t| t.value).collect();
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_simple_entry() {
        let text = "@misc{citekey, author=\"foo\", title = { bar }}";
        let expected = vec![
            Token::Special(Special::At),
            Token::Value("misc".to_string()),
            Token::Special(Special::BraceLeft),
            Token::Value("citekey".to_string()),
            Token::Special(Special::Comma),
            Token::Whitespace(Whitespace::Space),
            Token::Value("author".to_string()),
            Token::Special(Special::Equals),
            Token::Special(Special::Quote),
            Token::Value("foo".to_string()),
            Token::Special(Special::Quote),
            Token::Special(Special::Comma),
            Token::Whitespace(Whitespace::Space),
            Token::Value("title".to_string()),
            Token::Whitespace(Whitespace::Space),
            Token::Special(Special::Equals),
            Token::Whitespace(Whitespace::Space),
            Token::Special(Special::BraceLeft),
            Token::Whitespace(Whitespace::Space),
            Token::Value("bar".to_string()),
            Token::Whitespace(Whitespace::Space),
            Token::Special(Special::BraceRight),
            Token::Special(Special::BraceRight),
        ];
        let mut tokenizer = Tokenizer::new(text.chars());
        let tokens: Vec<Token> = tokenizer.tokenize().into_iter().map(|t| t.value).collect();
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_stringify() {
        let tokens = vec![
            Token::Special(Special::Quote),
            Token::Value("foo".to_string()),
            Token::Special(Special::Quote),
        ];
        assert_eq!(stringify(tokens), "\"foo\"");
    }
}
