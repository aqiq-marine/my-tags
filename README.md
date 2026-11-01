# my-tags

A small Rust implementation of a language for ontology-aware video annotations.

The v0.1 language separates three concerns:

1. **Types** — concepts and properties, declared with `def`.
2. **Instances** — named values of types, declared with `let`.
3. **Statements** — independent tag assertions about videos, declared with `stmt`.

## Current implementation

The initial Rust foundation includes:

- Lexer based on [logos](https://docs.rs/logos/).
- Hand-written recursive-descent parser.
- AST for type definitions, instance definitions, statements, and nested values.
- Initial semantic validation for names, single inheritance, Tag eligibility, Video subjects, and property value types.
- A `my-tags check <file>` CLI command.

## Quick start

```sh
cargo run -- check examples/basic.mt
```

Expected output:

```text
examples/basic.mt: valid (11 top-level items)
```

## Design principles

- `Tag` and `Number` are built-in types; neither can be redefined.
- `Tag` marks instances eligible to be used as tags; not every instance is a tag.
- `Video` is a user-defined type used to identify valid statement subjects.
- Types support single inheritance and intersection expressions (`+`).
- Properties are optional and have at most one value per statement.
- Statements are independent; repeated subject/tag pairs are retained separately.
- v0.1 annotates whole videos identified by stable instance IDs.
- Source files are the storage format. Querying is planned through a programmatic API; a query DSL is out of scope.

## Documentation

- [MVP Scope](docs/mvp-scope.md)
- [Language Specification v0.1](docs/language-v0.1.md)
- [Design Overview](docs/overview.md)
- [ADR 0001: Initial Language Model](docs/adr/0001-initial-language-model.md)

## Status

The parser and first semantic checks are an implementation foundation, not yet a complete v0.1 release. Query API and robust multi-file loading remain future work.
