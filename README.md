# my-tags

A small Rust implementation of a typed language for structured annotations.

The language separates three concerns:

1. **Types** — concepts and properties, declared with **def**.
2. **Instances** — named values of types, declared with **let**.
3. **Statements** — independent assertions connecting a subject to a tag instance, declared with **stmt**.

A statement subject can be any instance whose type inherits from the built-in **Subject** type. Videos are one possible subject kind, but the model is not video-specific.

## Current implementation

- Lexer based on logos and a hand-written recursive-descent parser.
- AST and semantic validation for type definitions, instances, statements, inheritance, tag eligibility, and property value types.
- ID-based Resolved Model with independent statements and nested anonymous values.
- Programmatic query API for tag and property filtering.
- Query DSL with type filters, property predicates, boolean composition, and set operators.
- Recursive query patterns for nested anonymous property values.
- CLI commands: my-tags check <file> validates a source file; my-tags run <file> evaluates its #show query directives.

## Built-in types

- **Subject**: base type required for statement subjects.
- **Tag**: marks instances eligible to be used as statement tags.
- **Float**: finite floating-point values.
- **Int**: signed integers.
- **Nat**: nonnegative integers.

Numeric types form the hierarchy Nat <: Int <: Float. Assignments and comparisons allow implicit widening from Nat to Int and from Int to Float; narrowing conversions are rejected. Float literals must be finite, so NaN and infinities are not supported.

Nonnegative integer literals are parsed as Nat, negative integer literals as Int, and decimal or exponent-form literals as Float. An integer literal can be assigned to a Float property through implicit widening.

## Quick start

Validate the basic example:

~~~sh
cargo run -- check examples/basic.mt
~~~

Run the query example:

~~~sh
cargo run -- run examples/query.mt
~~~

A query can combine numeric filtering with a recursively nested anonymous-value pattern:

~~~text
#show query Reaction {
  yield: (>= 80)
} & Hoge {
  huga: SolventInfo {
    solvent: methanol
  }
}
~~~

Within a nested pattern, a bare identifier means reference equality. For example, solvent: methanol is equivalent to solvent: (== methanol).

## Design principles

- Subject is the required base type for statement subjects; application-specific subject types inherit from it.
- Tag marks instances eligible to be used as tags; not every instance is a tag.
- Types support single inheritance and intersection expressions (+).
- Numeric types are Nat, Int, and Float, with implicit widening and finite Float values.
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
