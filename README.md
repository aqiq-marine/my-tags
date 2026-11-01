# my-tags

A language and data model for describing videos with structured, ontology-aware tags.

The v0.1 language separates three concerns:

1. **Types** — concepts and properties, declared with `def`.
2. **Instances** — named values of types, declared with `let`.
3. **Statements** — independent tag assertions about videos, declared with `stmt`.

## Design principles

- `Tag` marks instances eligible to be used as tags; not every instance is a tag.
- `Number` is a special primitive type.
- Types may use single inheritance and intersection expressions (`+`).
- Properties are optional and have at most one value per statement.
- Statements are independent; repeated subject/tag pairs are retained separately.
- v0.1 annotates whole videos identified by stable IDs.
- Data is stored in source files and searched through a programmatic API; there is no query DSL in v0.1.

## Documentation

- [MVP Scope](docs/mvp-scope.md)
- [Language Specification v0.1](docs/language-v0.1.md)
- [Design Overview](docs/overview.md)
- [ADR 0001: Initial Language Model](docs/adr/0001-initial-language-model.md)

## Status

The repository records the agreed v0.1 design. Implementation has not yet started.
