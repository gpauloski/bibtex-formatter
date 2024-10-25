# bibtex-formatter

A bibtex parser and formatter written in Rust.

## TODO

- [x] Add custom error types for common cases (end of stream, unexpected token).
- [x] Add parser tests for error modes.
- [x] Support commas and spaces in tag values.
- [x] Improve stringification.
- [x] Add CLI that can parse and print parsed entries to stdout.
- [ ] Create formatter that writes parsed entries.
  - [ ] Change Entry.tag from Vec<Tag> to HashMap<String>
  - [ ] Custom sort tags with title/author first
  - [ ] Lowercase tags, key, and kind
  - [ ] Sort entries by key before printing
  - [ ] Add write to file option
  - [ ] Add `--preview` flag
- [ ] Add end to end tests.
- [x] Refactor lexer to maintain state so line/column can be stored in Tokens.
- [ ] Support non-reference entry types (e.g., `@comment`).
