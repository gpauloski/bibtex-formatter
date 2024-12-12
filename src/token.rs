use std::fmt;
use std::iter::Peekable;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Special {
    At,
    BraceLeft,
    BraceRight,
    Comma,
    Equals,
    Pound,
    Quote,
}

impl Special {
    pub const fn from(c: &char) -> Option<Self> {
        match c {
            '@' => Some(Self::At),
            '{' => Some(Self::BraceLeft),
            '}' => Some(Self::BraceRight),
            ',' => Some(Self::Comma),
            '=' => Some(Self::Equals),
            '#' => Some(Self::Pound),
            '"' => Some(Self::Quote),
            _ => None,
        }
    }

    pub const fn is_special(c: &char) -> bool {
        matches!(c, '@' | '{' | '}' | ',' | '=' | '#' | '"')
    }

    pub const fn as_char(&self) -> char {
        match self {
            Self::At => '@',
            Self::BraceLeft => '{',
            Self::BraceRight => '}',
            Self::Comma => ',',
            Self::Equals => '=',
            Self::Pound => '#',
            Self::Quote => '"',
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Whitespace {
    NewLine,
    Space,
    Tab,
}

impl Whitespace {
    pub fn from(c: &char) -> Option<Self> {
        match c {
            '\n' | '\r' => Some(Self::NewLine),
            '\t' => Some(Self::Tab),
            c if c.is_whitespace() => Some(Self::Space),
            _ => None,
        }
    }

    pub const fn as_char(&self) -> char {
        match self {
            Self::NewLine => '\n',
            Self::Space => ' ',
            Self::Tab => '\t',
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
    Special(Special),
    Value(String),
    Whitespace(Whitespace),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::Special(c) => write!(f, "{}", c.as_char()),
            Self::Value(s) => write!(f, "{s}"),
            Self::Whitespace(c) => write!(f, "{}", c.as_char()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

impl Position {
    pub const fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenInfo {
    pub value: Token,
    pub position: Position,
}

impl TokenInfo {
    pub const fn new(value: Token, position: Position) -> Self {
        Self { value, position }
    }

    pub const fn is_special(&self) -> bool {
        matches!(self.value, Token::Special(_))
    }

    pub const fn is_value(&self) -> bool {
        matches!(self.value, Token::Value(_))
    }

    pub const fn is_whitespace(&self) -> bool {
        matches!(self.value, Token::Whitespace(_))
    }
}

pub struct Tokenizer<I>
where
    I: Iterator<Item = char>,
{
    stream: Peekable<I>,
    last: Position,
    next: Position,
}

impl<I: Iterator<Item = char>> Tokenizer<I> {
    pub fn new(iter: I) -> Self {
        Self {
            stream: iter.peekable(),
            last: Position { line: 1, column: 1 },
            next: Position { line: 1, column: 1 },
        }
    }

    pub fn peek(&mut self) -> Option<&char> {
        self.stream.peek()
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<char> {
        if let Some(next_char) = self.stream.next() {
            self.last.line = self.next.line;
            self.last.column = self.next.column;

            if matches!(next_char, '\n' | '\r') {
                self.next.line += 1;
                self.next.column = 1;
            } else {
                self.next.column += 1;
            }

            Some(next_char)
        } else {
            None
        }
    }

    pub fn tokenize(&mut self) -> Vec<TokenInfo> {
        let mut tokens: Vec<TokenInfo> = Vec::new();

        while let Some(c) = self.next() {
            let token = if let Some(token_type) = Special::from(&c) {
                TokenInfo::new(Token::Special(token_type), self.last)
            } else if let Some(token_type) = Whitespace::from(&c) {
                TokenInfo::new(Token::Whitespace(token_type), self.last)
            } else {
                let mut value = String::new();
                let position = self.last;

                value.push(c);
                while let Some(c) = self.peek() {
                    if Special::is_special(c) || c.is_whitespace() {
                        break;
                    }
                    if let Some(c) = self.next() {
                        value.push(c);
                    }
                }

                TokenInfo::new(Token::Value(value), position)
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
        let text = "@misc{citekey,\n  author=\"foo\", \ntitle = { bar }\n}";
        let expected = vec![
            TokenInfo::new(Token::Special(Special::At), Position::new(1, 1)),
            TokenInfo::new(Token::Value("misc".to_string()), Position::new(1, 2)),
            TokenInfo::new(Token::Special(Special::BraceLeft), Position::new(1, 6)),
            TokenInfo::new(Token::Value("citekey".to_string()), Position::new(1, 7)),
            TokenInfo::new(Token::Special(Special::Comma), Position::new(1, 14)),
            TokenInfo::new(Token::Whitespace(Whitespace::NewLine), Position::new(1, 15)),
            TokenInfo::new(Token::Whitespace(Whitespace::Space), Position::new(2, 1)),
            TokenInfo::new(Token::Whitespace(Whitespace::Space), Position::new(2, 2)),
            TokenInfo::new(Token::Value("author".to_string()), Position::new(2, 3)),
            TokenInfo::new(Token::Special(Special::Equals), Position::new(2, 9)),
            TokenInfo::new(Token::Special(Special::Quote), Position::new(2, 10)),
            TokenInfo::new(Token::Value("foo".to_string()), Position::new(2, 11)),
            TokenInfo::new(Token::Special(Special::Quote), Position::new(2, 14)),
            TokenInfo::new(Token::Special(Special::Comma), Position::new(2, 15)),
            TokenInfo::new(Token::Whitespace(Whitespace::Space), Position::new(2, 16)),
            TokenInfo::new(Token::Whitespace(Whitespace::NewLine), Position::new(2, 17)),
            TokenInfo::new(Token::Value("title".to_string()), Position::new(3, 1)),
            TokenInfo::new(Token::Whitespace(Whitespace::Space), Position::new(3, 6)),
            TokenInfo::new(Token::Special(Special::Equals), Position::new(3, 7)),
            TokenInfo::new(Token::Whitespace(Whitespace::Space), Position::new(3, 8)),
            TokenInfo::new(Token::Special(Special::BraceLeft), Position::new(3, 9)),
            TokenInfo::new(Token::Whitespace(Whitespace::Space), Position::new(3, 10)),
            TokenInfo::new(Token::Value("bar".to_string()), Position::new(3, 11)),
            TokenInfo::new(Token::Whitespace(Whitespace::Space), Position::new(3, 14)),
            TokenInfo::new(Token::Special(Special::BraceRight), Position::new(3, 15)),
            TokenInfo::new(Token::Whitespace(Whitespace::NewLine), Position::new(3, 16)),
            TokenInfo::new(Token::Special(Special::BraceRight), Position::new(4, 1)),
        ];
        let mut tokenizer = Tokenizer::new(text.chars());
        let tokens: Vec<TokenInfo> = tokenizer.tokenize();

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
