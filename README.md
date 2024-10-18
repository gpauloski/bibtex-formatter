# bibtex-formatter

A bibtex parser and formatter written in Rust.

## TODO

- [x] Add custom error types for common cases (end of stream, unexpected token).
- [x] Add parser tests for error modes.
- [x] Support commas and spaces in tag values.
- [ ] Add end to end tests.
- [ ] Improve stringification.
- [ ] Refactor lexer to maintain state so line/column can be stored in Tokens.
- [ ] Add CLI that can parse and print parsed entries to stdout.
- [ ] Create formatter that writes parsed entries.
- [ ] Support non-reference entry types (e.g., `@comment`).
