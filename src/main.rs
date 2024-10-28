mod error;
mod format;
mod models;
mod parse;
mod token;

pub use self::error::{Error, Result};
use format::{print_entries, write_entries};
use token::Tokenizer;

use clap::Parser;
use std::fs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input bibtex file
    #[arg(short, long)]
    input: String,
    /// Output bibtex file
    #[arg(short, long)]
    output: String,
    /// Preview formatted bibtex without writing
    #[arg(short, long, action)]
    preview: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let raw_bibtex = fs::read_to_string(args.input)?;

    let mut tokenizer = Tokenizer::new(raw_bibtex.chars());
    let tokens = tokenizer.tokenize();

    let mut parser = parse::Parser::new(tokens.into_iter());
    let mut parsed = parser.parse()?;

    parsed.sort();

    if args.preview {
        print_entries(&parsed);
    } else {
        write_entries(&parsed, &args.output)?;
    }
    Ok(())
}
