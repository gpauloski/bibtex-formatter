#![cfg(test)]
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// Integration tests that drive the built binary (via the CARGO_BIN_EXE_ env var
// Cargo sets for integration tests) to exercise --write/--check, exit codes, and
// in-place file IO, none of which the library-level snippet tests cover.

const EXIT_REFORMATTED: i32 = 5;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_bibtex-format"))
}

/// A unique scratch directory for a test, removed and recreated on each run.
fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("bibtex-format-cli-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn snippet(name: &str, kind: &str) -> String {
    fs::read_to_string(format!("tests/snippets/{name}.{kind}.bib")).unwrap()
}

#[test]
fn check_reports_unformatted_without_modifying() {
    let dir = scratch_dir("check-unformatted");
    let file = dir.join("refs.bib");
    let input = snippet("sort-tags", "in");
    fs::write(&file, &input).unwrap();

    let status = bin().arg("--check").arg(&file).status().unwrap();

    assert_eq!(status.code(), Some(EXIT_REFORMATTED));
    // --check never writes.
    assert_eq!(fs::read_to_string(&file).unwrap(), input);
}

#[test]
fn write_reformats_in_place() {
    let dir = scratch_dir("write-reformats");
    let file = dir.join("refs.bib");
    fs::write(&file, snippet("sort-tags", "in")).unwrap();

    let status = bin().arg("--write").arg(&file).status().unwrap();

    assert_eq!(status.code(), Some(EXIT_REFORMATTED));
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        snippet("sort-tags", "out")
    );
}

#[test]
fn write_then_check_are_idempotent() {
    let dir = scratch_dir("idempotent");
    let file = dir.join("refs.bib");
    fs::write(&file, snippet("sort-tags", "in")).unwrap();

    assert_eq!(
        bin().arg("--write").arg(&file).status().unwrap().code(),
        Some(EXIT_REFORMATTED)
    );
    // Second --write and a --check on the now-formatted file are no-ops.
    assert_eq!(
        bin().arg("--write").arg(&file).status().unwrap().code(),
        Some(0)
    );
    assert_eq!(
        bin().arg("--check").arg(&file).status().unwrap().code(),
        Some(0)
    );
}

#[test]
fn write_processes_multiple_files() {
    let dir = scratch_dir("multiple-files");
    let a = dir.join("a.bib");
    let b = dir.join("b.bib");
    fs::write(&a, snippet("sort-tags", "in")).unwrap();
    fs::write(&b, snippet("sort-entries", "in")).unwrap();

    let status = bin().arg("--write").arg(&a).arg(&b).status().unwrap();

    assert_eq!(status.code(), Some(EXIT_REFORMATTED));
    assert_eq!(fs::read_to_string(&a).unwrap(), snippet("sort-tags", "out"));
    assert_eq!(
        fs::read_to_string(&b).unwrap(),
        snippet("sort-entries", "out")
    );
}
