#![cfg(test)]
use test_case::test_case;

use bibtex_format::format::Formatter;
use bibtex_format::parse::Parser;
use bibtex_format::token::Tokenizer;
use bibtex_format::Result;
use std::fs;

// The test_case macros are used to run a test for each pair of input/output
// files in tests/snippets/. Each input/output pair takes the form:
//     tests/snippets/{test-name}.{in|out}.bib
//
// For example, to add a new test case called foo-bar:
//   1. Create a bibtex file tests/snippets/foo-bar.in.bib containing the
//      input bibtex entries to be formatted.
//   2. Create a bibtex file tests/snippets/foo-bar.out.bib containing the
//      expected formatting of the input file.
//   3. Add a new test case macro to the function whose formatter config the
//      snippet should run under, with the test name and a short description:
//          #[test_case("foo-bar" ; "compare foo and bar")
//      Please keep the test cases sorted by test name within each function.
//        - validate_snippets: the default formatter (sorted).
//        - validate_snippets_skip_sort: --skip-sort-entries (preserves order
//          and the original whitespace between elements).
//        - validate_snippets_remove_comments: --remove-comments.
//
// Notes:
//   - Leading and trailing whitespace is trimmed from the expected output, so
//     only whitespace *between* elements is asserted.
#[test_case("coalesce-multiline-content" ; "coalesce mutliline contents")]
#[test_case("comment-nested-braces" ; "round-trip @comment with nested braces")]
#[test_case("comment-travels-with-entry" ; "comment moves with its entry when sorted")]
#[test_case("implicit-comments" ; "attach comments to following entry")]
#[test_case("non-delimited-content" ; "non-delimited single word contents")]
#[test_case("preserve-title-casing" ; "preserve title casing with braces")]
#[test_case("quotes-to-braces" ; "convert quotes to braces in tag contents")]
#[test_case("remove-empty-tags" ; "remove tags with empty content")]
#[test_case("sort-entries" ; "sort entries in file")]
#[test_case("sort-preamble-first"; "sort preambles at top of file")]
#[test_case("sort-tags" ; "sort tags in entry")]
#[test_case("string-concat" ; "format entries with string concatentation")]
#[test_case("string-entries" ; "format string entry types")]
#[test_case("trailing-comments-without-format" ; "keep trailing comments in order without formatting")]
fn validate_snippets(name: &str) -> Result<()> {
    run_snippet(name, &Formatter::builder().build())
}

// Snippets exercising --skip-sort-entries, which preserves the source order and
// the original whitespace between every element. Keep sorted by test name.
#[test_case("skip-sort-comments" ; "preserve mixed comment layout without sorting")]
#[test_case("skip-sort-spacing" ; "preserve entry spacing verbatim without sorting")]
fn validate_snippets_skip_sort(name: &str) -> Result<()> {
    run_snippet(name, &Formatter::builder().sort_entries(false).build())
}

// Snippets exercising --remove-comments. Keep sorted by test name.
#[test_case("remove-comments" ; "strip all comments before sorting")]
fn validate_snippets_remove_comments(name: &str) -> Result<()> {
    run_snippet(name, &Formatter::builder().remove_comments(true).build())
}

// Snippets exercising --remove-duplicates, which collapses exact-duplicate
// entries and keeps entries that share a key but differ. Keep sorted by name.
#[test_case("remove-duplicates" ; "collapse exact duplicates, keep conflicts")]
fn validate_snippets_remove_duplicates(name: &str) -> Result<()> {
    run_snippet_deduped(name, &Formatter::builder().build())
}

fn run_snippet(name: &str, formatter: &Formatter) -> Result<()> {
    run_snippet_inner(name, formatter, false)
}

fn run_snippet_deduped(name: &str, formatter: &Formatter) -> Result<()> {
    run_snippet_inner(name, formatter, true)
}

fn run_snippet_inner(name: &str, formatter: &Formatter, dedup: bool) -> Result<()> {
    let input = format!("tests/snippets/{}.in.bib", name);
    let output = format!("tests/snippets/{}.out.bib", name);

    let raw_input = fs::read_to_string(&input)?;
    let expected = fs::read_to_string(&output)?;

    let mut tokenizer = Tokenizer::new(raw_input.chars());
    let tokens = tokenizer.tokenize();
    let mut parser = Parser::new(tokens.into_iter());
    let mut entries = parser.parse()?;

    if dedup {
        entries.remove_duplicates();
    }

    let formatted = formatter.format_entries(&entries);

    assert_eq!(formatted, expected.trim());

    Ok(())
}
