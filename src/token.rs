#[derive(Clone, Debug, PartialEq)]
pub enum Whitespace {
    NewLine,
    Space,
    Tab,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    At,
    BraceLeft,
    BraceRight,
    Comma,
    Equals,
    Value(String),
    Quote,
    Whitespace(Whitespace),
}

impl Token {
    pub fn is_special(c: &char) -> bool {
        matches!(c, '@' | '{' | '}' | ',' | '=' | '"')
    }

    pub fn is_whitespace(&self) -> bool {
        matches!(self, Token::Whitespace(_))
    }
}

pub fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut stream = text.chars().peekable();

    while let Some(c) = stream.next() {
        let token = match c {
            '\n' | '\r' => Token::Whitespace(Whitespace::NewLine),
            '\t' => Token::Whitespace(Whitespace::Tab),
            c if c.is_whitespace() => Token::Whitespace(Whitespace::Space),
            '@' => Token::At,
            '{' => Token::BraceLeft,
            '}' => Token::BraceRight,
            ',' => Token::Comma,
            '=' => Token::Equals,
            '"' => Token::Quote,
            _ => {
                let mut value = String::new();
                value.push(c);
                while let Some(c) = stream.peek() {
                    if Token::is_special(c) || c.is_whitespace() {
                        break;
                    } else {
                        value.push(stream.next().unwrap())
                    }
                }
                Token::Value(value)
            }
        };
        tokens.push(token);
    }

    tokens
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
            Token::Value(s) => string.push_str(&s),
            _ => {
                let c = match token {
                    Token::At => '@',
                    Token::BraceLeft => '{',
                    Token::BraceRight => '}',
                    Token::Comma => ',',
                    Token::Equals => '=',
                    Token::Quote => '"',
                    Token::Whitespace(Whitespace::NewLine) => '\n',
                    Token::Whitespace(Whitespace::Space) => ' ',
                    Token::Whitespace(Whitespace::Tab) => '\t',
                    // TODO: fix this type narrowing.
                    _ => panic!("Unreachable!"),
                };
                string.push(c);
            }
        }
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
        assert_eq!(tokenize(text), expected);
    }

    #[test]
    fn test_simple_entry() {
        let text = "@misc{citekey, author=\"foo\", title = { bar }}";
        let expected = vec![
            Token::At,
            Token::Value("misc".to_string()),
            Token::BraceLeft,
            Token::Value("citekey".to_string()),
            Token::Comma,
            Token::Whitespace(Whitespace::Space),
            Token::Value("author".to_string()),
            Token::Equals,
            Token::Quote,
            Token::Value("foo".to_string()),
            Token::Quote,
            Token::Comma,
            Token::Whitespace(Whitespace::Space),
            Token::Value("title".to_string()),
            Token::Whitespace(Whitespace::Space),
            Token::Equals,
            Token::Whitespace(Whitespace::Space),
            Token::BraceLeft,
            Token::Whitespace(Whitespace::Space),
            Token::Value("bar".to_string()),
            Token::Whitespace(Whitespace::Space),
            Token::BraceRight,
            Token::BraceRight,
        ];
        assert_eq!(tokenize(text), expected);
    }

    #[test]
    fn test_stringify() {
        let tokens = vec![Token::Quote, Token::Value("foo".to_string()), Token::Quote];
        assert_eq!(stringify(tokens), "\"foo\"");
    }
}
