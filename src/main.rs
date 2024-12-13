use bibtex_format::format::Formatter;
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
    /// Skip sorting entries.
    #[arg(long)]
    skip_sort_entries: bool,
    /// Skip sorting tags.
    #[arg(long)]
    skip_sort_tags: bool,
    /// Skip formatting titles.
    #[arg(long)]
    skip_title_format: bool,
    /// Retain tags with empty contents.
    #[arg(long)]
    retain_empty_tags: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let raw_bibtex = match fs::read_to_string(args.input) {
        Ok(raw) => raw,
        Err(error) => {
            println!("Error parsing input file: {error}");
            return ExitCode::from(1);
        }
    };

    let mut tokenizer = Tokenizer::new(raw_bibtex.chars());
    let tokens = tokenizer.tokenize();

    let mut parser = parse::Parser::new(tokens.into_iter());
    let entries = match parser.parse() {
        Ok(entries) => entries,
        Err(error) => {
            println!("{error}");
            return ExitCode::from(2);
        }
    };

    let formatter = Formatter::builder()
        .format_title(!args.skip_title_format)
        .skip_empty_tags(!args.retain_empty_tags)
        .sort_entries(!args.skip_sort_entries)
        .sort_tags(!args.skip_sort_tags)
        .build();

    if let Some(output) = args.output {
        let result = formatter.write_entries(&entries, &output);
        if let Err(error) = result {
            println!("Error parsing output file: {error}");
            return ExitCode::from(3);
        }
    } else {
        println!("{}", formatter.format_entries(&entries));
    }

    ExitCode::SUCCESS
}
