# bibtex-formatter

An opinionated bibtex parser and formatter written in Rust.

> [!WARNING]
> This is still being tested and has some known limitations (see below).
> Be careful to not overwrite your source bibtex.

> [!TIP]
> Please open an issue if you find an edge case or bug!

The following formatting rules are applied (based on my personal preference :)):
* Entry types, citation keys, and tag names are lowercase.
* Entries are sorted by citation key.
* The title and author tags are first in an entry followed by the remaining tags sorted by name.
* Braces are used for tag content rather than quotes.

Learn more about the bibtex format at [bibtex.org](https://www.bibtex.org/Format/).

**Limitations:** The following are untested/unsupported but will be eventually.
* `@STRING`, `@PREAMBLE`, and `@COMMENT` entry types.
* String concatentation (e.g., `title = "{Bib}" # "\TeX"`).

## Installation

Installation requires the Rust toolchain. Get it [here](https://www.rust-lang.org/tools/install).

**Clone the source:**
```bash
git clone git@github.com:gpauloski/bibtex-formatter.git
```

**Compile and install the binary:**
```bash
cargo install --path .
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
    year={2020}
    doi={10.1109/IPDPS47924.2020.00050},
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
    title = {TaPS: A Performance Evaluation Suite for Task-based Execution Frameworks},
    author = {Pauloski, J. Gregory and Hayot-Sasson, Valerie and Gonthier, Maxime and Hudson, Nathaniel and Pan, Haochen and Zhou, Sicheng and Foster, Ian and Chard, Kyle},
    address = {New York, NY, USA},
    booktitle = {IEEE 20th International Conference on e-Science},
    doi = {10.1109/e-Science62913.2024.10678702},
    pages = {1-10},
    publisher = {IEEE},
    year = {2024},
}

@inproceedings{zhang2020compressed,
    title = {{Efficient I/O for Neural Network Training with Compressed Data}},
    author = {Z. {Zhang} and L. {Huang} and J. G. {Pauloski} and I. T. {Foster}},
    booktitle = {2020 IEEE International Parallel and Distributed Processing Symposium (IPDPS)},
    doi = {10.1109/IPDPS47924.2020.00050},
    number = {},
    pages = {409-418},
    volume = {},
    year = {2020},
}
```

## TODO

- [x] Add custom error types for common cases (end of stream, unexpected token).
- [x] Add parser tests for error modes.
- [x] Support commas and spaces in tag values.
- [x] Improve stringification.
- [x] Add CLI that can parse and print parsed entries to stdout.
- [x] Create formatter that writes parsed entries.
  - [x] [Skip] Change Entry.tag from Vec<Tag> to HashMap<String>
  - [x] Custom sort tags with title/author first
  - [x] Lowercase tags, key, and kind
  - [x] Sort entries by key before printing
  - [x] Add write to file option
  - [x] Add `--preview` flag
- [x] Refactor lexer to maintain state so line/column can be stored in Tokens.
- [x] Update README with install, usage, and examples.
- [x] Improve display formatting of error types.
- [ ] Add more unit tests.
- [ ] Add end to end tests.
- [ ] Support non-reference entry types (e.g., `@comment`).
- [ ] Support string concatenation when tag content is quoted.
- [ ] Title formatting: insert `{}` around capitalized characters.
