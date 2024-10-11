mod error;
mod parse;
mod token;

pub use self::error::{Error, Result};

fn main() {
    let text = "@misc{hello, title={text}}";
    println!("Input: {}", text);
    let tokens = token::tokenize(text);
    println!("Output: {:?}", tokens);

    let mut parser = parse::Parser::new(tokens.into_iter());
    parser.parse();
}
