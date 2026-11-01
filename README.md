# my-tags

A small Rust implementation of a language for ontology-aware video annotations.

The v0.1 language separates three concerns:

1. **Types** — concepts and properties, declared with `def`.
2. **Instances** — named values of types, declared with `let`.
3. **Statements** — independent tag assertions about videos, declared with `stmt`.

The language also provides a query DSL. Queries select video subjects by tag type and statement-local property predicates, including recursively nested anonymous values.

## Current implementation

- Lexer based on [logos](https://docs.rs/logos/) and a hand-written recursive-descent parser.
- AST and semantic validation for type definitions, instances, statements, inheritance, tag eligibility, and property value types.
- ID-based Resolved Model with independent statements and nested anonymous values.
- Programmatic query API for tag and property filtering.
- Query DSL with type filters, property predicates, boolean composition, and set operators.
- Recursive query patterns for nested anonymous property values.
- CLI commands: `my-tags check <file>` validates a source file; `my-tags run <file>` evaluates its `#show query` directives.

## Quick start

Validate the basic data example:

```sh
cargo run -- check examples/basic.mt
```

Run the query example:

```sh
cargo run -- run examples/query.mt
```

The query example demonstrates filtering by a numeric property and matching a nested anonymous value:

```text
#show query Reaction {
  yield: (>= 80)
} & Hoge {
  huga: SolventInfo {
    solvent: methanol
  }
}
```

Within a nested pattern, a bare identifier means reference equality. For example, `solvent: methanol` is equivalent to `solvent: (== methanol)`.

## Design principles

- `Tag` and `Number` are built-in types; neither can be redefined.
- `Tag` marks instances eligible to be used as tags; not every instance is a tag.
- `Video` is a user-defined type used to identify valid statement subjects.
- Types support single inheritance and intersection expressions (`+`).
- Properties are optional and have at most one value per statement.
- Statements are independent; repeated subject/tag pairs are retained separately.
- Nested anonymous instances can be stored as property values and matched recursively by the query DSL.
- Query conditions in one type block apply to the same statement. Matching subjects are combined by set operators.
- Query evaluation currently scans statements; indexing is deferred.
- Source files are the canonical storage format.

## Documentation

- [MVP Scope](docs/mvp-scope.md)
- [Language Specification v0.1](docs/language-v0.1.md)
- [Resolved Model](docs/resolved-model.md)
- [Design Overview](docs/overview.md)
- [ADR 0001: Initial Language Model](docs/adr/0001-initial-language-model.md)

## Status

The parser, semantic validation, Resolved Model, programmatic query API, and query DSL are implemented with CI coverage. Multi-file loading, query indexing, and a stable interchange format remain future work.
