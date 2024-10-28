# bibtex-formatter

A bibtex parser and formatter written in Rust.

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
- [ ] Add end to end tests.
- [x] Refactor lexer to maintain state so line/column can be stored in Tokens.
- [ ] Update README with install, usage, and examples.
- [ ] Improve display formatting of error types.
- [ ] Support non-reference entry types (e.g., `@comment`).
