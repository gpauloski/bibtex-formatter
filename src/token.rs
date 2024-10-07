// use crate::Result;

#[derive(Debug, PartialEq)]
pub enum Token {
    At,
    BraceLeft,
    BraceRight,
    Comma,
    Percent,
    Equals,
    Value(String),
    Quote,
}

impl Token {
    fn is_special(c: &char) -> bool {
        matches!(c, '@' | '{' | '}' | ',' | '=' | '%' | '"')
    }
}

pub fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut stream = text.chars().peekable();

    while let Some(c) = stream.next() {
        let token = match c {
            '\n' => continue,
            c if c.is_whitespace() => continue,
            '@' => Token::At,
            '{' => Token::BraceLeft,
            '}' => Token::BraceRight,
            ',' => Token::Comma,
            '%' => Token::Percent,
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
            Token::Value("author".to_string()),
            Token::Equals,
            Token::Quote,
            Token::Value("foo".to_string()),
            Token::Quote,
            Token::Comma,
            Token::Value("title".to_string()),
            Token::Equals,
            Token::BraceLeft,
            Token::Value("bar".to_string()),
            Token::BraceRight,
            Token::BraceRight,
        ];
        assert_eq!(tokenize(text), expected);
    }
}
