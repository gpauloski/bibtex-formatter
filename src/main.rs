use bibtex_format::format::{print_entries, write_entries};
use bibtex_format::parse;
use bibtex_format::token::Tokenizer;

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
    /// Write formatted bibtex to this file.
    #[arg(short, long)]
    output: Option<String>,
    /// Retain tags with empty contents.
    #[arg(short, long)]
    retain_empty_tags: bool,
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

    let mut parser = parse::Parser::new(tokens.into_iter(), !args.retain_empty_tags);
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
