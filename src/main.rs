mod error;
mod parse;
mod token;

pub use self::error::{Error, Result};

fn main() {
    let text = "@misc{hello, title={This is a title}}";
    println!("Input: {}", text);
    let tokens = token::tokenize(text);
    println!("Output: {:?}", tokens);

    let mut parser = parse::Parser::new(tokens.into_iter());
    let parsed = parser.parse();
    println!("Parsed: {:?}", parsed);
}
