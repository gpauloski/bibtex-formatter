#![cfg(test)]
use test_case::test_case;

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
//   3. Add a new test case macro to this function with the test name and
//      short description of the test.
//          #[test_case("foo-bar" ; "compare foo and bar")
//      Please keep the test cases sorted by test name.
//
// Notes:
//   - Leading and trailing whitespace is trimmed from the expected output.
#[test_case("quotes-to-braces" ; "convert quotes to braces in tag contents")]
#[test_case("sort-entries" ; "sort entries in file")]
#[test_case("sort-tags" ; "sort tags in entry")]
fn validate_snippets(name: &str) -> Result<()> {
    let input = format!("tests/snippets/{}.in.bib", name);
    let output = format!("tests/snippets/{}.out.bib", name);

    let raw_input = fs::read_to_string(&input)?;
    let expected = fs::read_to_string(&output)?;

    let mut tokenizer = Tokenizer::new(raw_input.chars());
    let tokens = tokenizer.tokenize();
    let mut parser = Parser::new(tokens.into_iter());
    let mut input_entries = parser.parse()?;
    input_entries.sort();

    let formatted = input_entries
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<String>>()
        .join("\n\n");

    assert_eq!(formatted, expected.trim());

    Ok(())
}