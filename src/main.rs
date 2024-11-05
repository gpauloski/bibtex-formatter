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
use std::process::ExitCode;

/// Parse and format bibtex files.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input bibtex file.
    #[arg()]
    input: String,
    /// Output bibtex file. Formatted bibtex is printed to stdout unless this
    /// option is provided.
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let raw_bibtex = match fs::read_to_string(args.input) {
        Ok(raw) => raw,
        Err(error) => {
            println!("Error parsing input file: {}", error);
            return ExitCode::from(1);
        }
    };

    let mut tokenizer = Tokenizer::new(raw_bibtex.chars());
    let tokens = tokenizer.tokenize();

    let mut parser = parse::Parser::new(tokens.into_iter());
    let mut parsed = match parser.parse() {
        Ok(parsed) => parsed,
        Err(error) => {
            println!("{}", error);
            return ExitCode::from(2);
        }
    };

    parsed.sort();

    if let Some(output) = args.output {
        let result = write_entries(&parsed, &output);
        if let Err(error) = result {
            println!("Error parsing output file: {}", error);
            return ExitCode::from(3);
        }
    } else {
        print_entries(&parsed);
    }

    ExitCode::SUCCESS
}
