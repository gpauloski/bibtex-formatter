mod format;
mod error;
mod parse;
mod token;

pub use self::error::{Error, Result};
use format::print_entries;
use token::Tokenizer;

use clap::Parser;
use std::fs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input bibtex file
    #[arg(short, long)]
    input: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let raw_bibtex = fs::read_to_string(args.input)?;

    let mut tokenizer = Tokenizer::new(raw_bibtex.chars());
    let tokens = tokenizer.tokenize();

    let mut parser = parse::Parser::new(tokens.into_iter());
    let parsed = parser.parse()?;

    print_entries(&parsed);
    Ok(())
}
