# bibtex-formatter

An opinionated bibtex parser and formatter written in Rust.

> [!WARNING]
> While I strive to write perfect code :), there might be edge cases
> so be careful to not overwrite your source bibtex.

> [!TIP]
> Please open an issue if you find an edge case or bug!

The following formatting rules are applied by default (based on my personal preference :)):
* Entry types, citation keys, and tag names are lowercase.
* Entries are sorted by citation key.
* The title and author tags are first in an entry followed by the remaining tags sorted by name.
* Braces are used for tag content rather than quotes.
* Capitalized words in title tags are wrapped in braces to preserve formatting.
* Comments—both `@comment{...}` entries and free text between entries—are preserved, attach to the entry that follows them, and move with it when entries are sorted; comments after the last entry stay at the end.

Most rules are configurable; see `--help`. Learn more about the bibtex format at [bibtex.org](https://www.bibtex.org/Format/) and in this [nice summary](https://maverick.inria.fr/~Xavier.Decoret/resources/xdkbibtex/bibtex_summary.html).

## Installation

Installation requires the Rust toolchain. Get it [here](https://www.rust-lang.org/tools/install).

Install the binary directly from GitHub:
```bash
cargo install --git https://github.com/gpauloski/bibtex-formatter.git
```

## Usage

Print formatted bibtex to stdout:
```bash
bibtex-format references.bib
```

Write formatted bibtex to a new file:
```bash
bibtex-format references.bib --output formatted.bib
```

Strip all comments (both `@comment{...}` entries and free text between entries):
```bash
bibtex-format references.bib --remove-comments
```

Collapse exact-duplicate entries (same cite key and content):
```bash
bibtex-format references.bib --remove-duplicates
```
Entries that share a cite key but differ in content are all kept; each such
collision is reported as a warning on stderr so nothing is dropped silently.

Reformat one or more files in place:
```bash
bibtex-format --write references.bib other.bib
```

Check whether files are formatted without modifying them (exits non-zero if any
file would change):
```bash
bibtex-format --check references.bib
```

Run `bibtex-format --help` to see all available options.

### Exit codes

| Code | Meaning |
| ---- | ------- |
| `0`  | Success; nothing needed reformatting. |
| `1`  | Could not read an input file. |
| `2`  | Invalid arguments. |
| `3`  | Failed to parse an input file. |
| `4`  | Failed to write an output file. |
| `5`  | A file was reformatted (`--write`) or would be reformatted (`--check`). |

## Use as a pre-commit hook

This repository provides [pre-commit](https://pre-commit.com) hooks so `.bib`
files stay formatted on every commit. Add the following to your
`.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/gpauloski/bibtex-formatter
    rev: v0.1.0
    hooks:
      - id: bibtex-format          # auto-fix in place
      # - id: bibtex-format-check  # verify only, no writes
```

The `bibtex-format` hook reformats staged `.bib` files in place and fails the
commit when it changes anything, so you can review and re-stage. Use
`bibtex-format-check` instead to fail without modifying files. Both build the
binary via pre-commit's Rust support, so the Rust toolchain must be available.

## Example

**Input:** `references.bib`
```bib
@inproceedings{zhang2020compressed,
    author={Z. {Zhang} and L. {Huang} and J. G. {Pauloski} and I. T. {Foster}},
    title={{Efficient I/O for Neural Network Training with Compressed Data}},
    booktitle={2020 IEEE International Parallel and Distributed Processing Symposium (IPDPS)},
    number={},
    pages={409-418},
    volume={},
    year={2020},
    doi={10.1109/IPDPS47924.2020.00050}
}

@INPROCEEDINGS{PAULOSKI2024TAPS,
    ADDRESS = "New York, NY, USA",
    AUTHOR = "Pauloski, J. Gregory and Hayot-Sasson, Valerie and Gonthier, Maxime and Hudson, Nathaniel and Pan, Haochen and Zhou, Sicheng and Foster, Ian and Chard, Kyle",
    BOOKTITLE = "IEEE 20th International Conference on e-Science",
    DOI = "10.1109/e-Science62913.2024.10678702",
    PAGES = "1-10",
    PUBLISHER = "IEEE",
    TITLE = "TaPS: A Performance Evaluation Suite for Task-based Execution Frameworks",
    YEAR = "2024"
}
```

**Output:** `formatted.bib`
```bib
@inproceedings{pauloski2024taps,
    title = {{TaPS}: {A} {P}erformance {E}valuation {S}uite for {T}ask-based {E}xecution {F}rameworks},
    author = {Pauloski, J. Gregory and Hayot-Sasson, Valerie and Gonthier, Maxime and Hudson, Nathaniel and Pan, Haochen and Zhou, Sicheng and Foster, Ian and Chard, Kyle},
    address = {New York, NY, USA},
    booktitle = {IEEE 20th International Conference on e-Science},
    doi = {10.1109/e-Science62913.2024.10678702},
    pages = {1-10},
    publisher = {IEEE},
    year = {2024},
}

@inproceedings{zhang2020compressed,
    title = {Efficient {I/O} for {N}eural {N}etwork {T}raining with {C}ompressed {D}ata},
    author = {Z. {Zhang} and L. {Huang} and J. G. {Pauloski} and I. T. {Foster}},
    booktitle = {2020 IEEE International Parallel and Distributed Processing Symposium (IPDPS)},
    doi = {10.1109/IPDPS47924.2020.00050},
    pages = {409-418},
    year = {2020},
}
```

## Developing

Clone the repository and build from source:
```bash
git clone git@github.com:gpauloski/bibtex-formatter.git
cd bibtex-formatter
cargo build
```

Run the formatter without installing:
```bash
cargo run -- references.bib
```

Run the test suite:
```bash
cargo test
```

Format and lint before opening a pull request:
```bash
cargo fmt
cargo clippy
```
