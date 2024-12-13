# bibtex-formatter

An opinionated bibtex parser and formatter written in Rust.

> [!WARNING]
> While I strive to write perfect code :), there might be edge cases
> so be careful to not overwrite your source bibtex.

> [!TIP]
> Please open an issue if you find an edge case or bug!

The following formatting rules are applied (based on my personal preference :)):
* Entry types, citation keys, and tag names are lowercase.
* Entries are sorted by citation key.
* The title and author tags are first in an entry followed by the remaining tags sorted by name.
* Braces are used for tag content rather than quotes.
* Capitalized words in title tags are wrapped in braces to preserve formatting.

Learn more about the bibtex format at [bibtex.org](https://www.bibtex.org/Format/) and in this [nice summary](https://maverick.inria.fr/~Xavier.Decoret/resources/xdkbibtex/bibtex_summary.html).

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
