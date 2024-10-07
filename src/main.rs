mod error;
mod token;

pub use self::error::{Error, Result};

fn main() {
    let text = "@misc{hello, title={text}}";
    println!("Input: {}", text);
    println!("Output: {:?}", token::tokenize(text));
}
