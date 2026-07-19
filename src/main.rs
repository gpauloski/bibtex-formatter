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
    /// Input bibtex file(s).
    #[arg(required = true)]
    inputs: Vec<String>,
    /// Write formatted bibtex to this file (single input only).
    #[arg(short, long, conflicts_with_all = ["write", "check"])]
    output: Option<String>,
    /// Reformat each input file in place.
    #[arg(short, long, conflicts_with = "check")]
    write: bool,
    /// Check that each input file is formatted without modifying it. Exits
    /// non-zero if any file would be reformatted.
    #[arg(long)]
    check: bool,
    /// Skip sorting entries.
    #[arg(long)]
    skip_sort_entries: bool,
    /// Skip sorting tags.
    #[arg(long)]
    skip_sort_tags: bool,
    /// Skip formatting titles.
    #[arg(long)]
    skip_title_format: bool,
    /// Remove tags with empty contents.
    #[arg(long)]
    remove_empty_tags: bool,
    /// Remove all comments (implicit text and @comment entries).
    #[arg(long)]
    remove_comments: bool,
    /// Collapse exact-duplicate entries (same key and content). Entries that
    /// share a key but differ are all kept and reported as warnings.
    #[arg(long)]
    remove_duplicates: bool,
}

/// Exit codes. pre-commit (and any caller) treats any non-zero code as failure.
/// Code 2 matches clap's own exit code for invalid arguments.
const EXIT_READ_ERROR: u8 = 1;
const EXIT_ARG_ERROR: u8 = 2;
const EXIT_PARSE_ERROR: u8 = 3;
const EXIT_WRITE_ERROR: u8 = 4;
const EXIT_REFORMATTED: u8 = 5;

fn main() -> ExitCode {
    let args = Args::parse();

    if args.output.is_some() && args.inputs.len() > 1 {
        eprintln!("Error: --output can only be used with a single input file.");
        return ExitCode::from(EXIT_ARG_ERROR);
    }

    let formatter = Formatter::builder()
        .format_title(!args.skip_title_format)
        .remove_comments(args.remove_comments)
        .skip_empty_tags(args.remove_empty_tags)
        .sort_entries(!args.skip_sort_entries)
        .sort_tags(!args.skip_sort_tags)
        .build();

    let mut reformatted = false;
    for input in &args.inputs {
        let formatted = match format_file(input, &formatter, args.remove_duplicates) {
            Ok(formatted) => formatted,
            Err(code) => return ExitCode::from(code),
        };

        if args.write || args.check {
            // In write/check mode files carry a single trailing newline so the
            // compare-and-rewrite is idempotent.
            let current = match fs::read_to_string(input) {
                Ok(current) => current,
                Err(error) => {
                    eprintln!("Error reading input file `{input}`: {error}");
                    return ExitCode::from(EXIT_READ_ERROR);
                }
            };
            if current == formatted {
                continue;
            }
            reformatted = true;
            if args.write {
                if let Err(error) = fs::write(input, &formatted) {
                    eprintln!("Error writing output file `{input}`: {error}");
                    return ExitCode::from(EXIT_WRITE_ERROR);
                }
                eprintln!("reformatted {input}");
            } else {
                eprintln!("would reformat {input}");
            }
        } else if let Some(output) = &args.output {
            if let Err(error) = fs::write(output, &formatted) {
                eprintln!("Error writing output file `{output}`: {error}");
                return ExitCode::from(EXIT_WRITE_ERROR);
            }
        } else {
            // format_file appends a trailing newline; print without adding
            // another.
            print!("{formatted}");
        }
    }

    if reformatted {
        ExitCode::from(EXIT_REFORMATTED)
    } else {
        ExitCode::SUCCESS
    }
}

/// Read, parse, and format a single file, returning its formatted contents with
/// a single trailing newline. On failure a message is printed and the matching
/// exit code is returned in `Err`.
fn format_file(input: &str, formatter: &Formatter, remove_duplicates: bool) -> Result<String, u8> {
    let raw_bibtex = match fs::read_to_string(input) {
        Ok(raw) => raw,
        Err(error) => {
            eprintln!("Error reading input file `{input}`: {error}");
            return Err(EXIT_READ_ERROR);
        }
    };

    let mut tokenizer = Tokenizer::new(raw_bibtex.chars());
    let tokens = tokenizer.tokenize();

    let mut parser = parse::Parser::new(tokens.into_iter());
    let mut entries = match parser.parse() {
        Ok(entries) => entries,
        Err(error) => {
            eprintln!("{error}");
            return Err(EXIT_PARSE_ERROR);
        }
    };

    if remove_duplicates {
        for warning in entries.remove_duplicates() {
            eprintln!("{warning}");
        }
    }

    Ok(format!("{}\n", formatter.format_entries(&entries)))
}
