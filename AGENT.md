# Agent Guide

This file provides working conventions for contributors and coding agents in this repository.

## Project overview

`my-tags` is a small Rust implementation of a typed language for structured annotations. The language has three main layers:

- `def`: type and property declarations
- `let`: named instances
- `stmt`: independent annotations connecting a subject to a tag instance

The parser, semantic analysis, ID-based resolved model, query API, query DSL, and multi-file imports are core parts of the project.

## Before making changes

1. Read the relevant parts of [Language Specification v0.1](docs/language-v0.1.md) and the relevant design records in [docs/adr](docs/adr/).
2. Inspect nearby implementation and tests before introducing a new abstraction or changing an existing behavior.
3. Check whether the proposed change affects syntax, typing, name resolution, module behavior, query semantics, or the resolved model. These areas often have implications beyond the file being edited.
4. Keep the implementation consistent with the specification. If implementation experience suggests that the specification or an existing decision should change, make that change explicit rather than silently diverging.

## Architecture and implementation conventions

- Keep parsing, module loading/name rewriting, semantic validation, resolution, and runtime/query behavior separated according to the existing architecture.
- Prefer stable IDs and explicit ownership/identity over relying on display names where the codebase already follows that model.
- Preserve useful diagnostics. Invalid or ambiguous input should produce an actionable error rather than silently choosing one interpretation.
- Treat module aliases and qualified names as scoped names; do not introduce implicit transitive visibility or name leakage without an explicit design decision.
- Add focused unit tests near the implementation and integration tests under `tests/` for user-visible language behavior.
- Update examples and documentation when a language feature or its observable behavior changes.
- Avoid unrelated refactoring in feature changes. Follow existing Rust style and keep public APIs intentional.

## Architecture Decision Records (ADRs)

**Write ADRs proactively.** Do not wait until a design becomes difficult to change or until someone asks for documentation.

Create or update an ADR when a change makes, revises, or meaningfully constrains a design decision, especially decisions involving:

- Language syntax or semantics
- Type-system rules, inheritance, intersections, or property resolution
- Name identity, namespaces, imports, and visibility
- AST, resolved-model, or query-model representation
- Public APIs, serialization, persistence, or compatibility
- Architectural boundaries, dependencies, or significant implementation trade-offs

A useful rule of thumb: if a future contributor might reasonably ask "why was it designed this way?" or if another plausible design was considered, record the decision.

### ADR workflow

1. Check `docs/adr/` for an existing record covering the decision.
2. If the decision is new, add the next sequentially numbered file, using a descriptive name (for example, `0004-qualified-property-references.md`).
3. Use the existing ADRs as a style reference. Include:
   - **Status** (for example, Proposed, Accepted, Superseded, or Deprecated)
   - **Date**
   - **Context** and the problem being addressed
   - **Decision**, stated precisely enough to guide implementation
   - **Consequences**, including costs and limitations
   - **Alternatives considered** and why they were not selected
4. Link related specifications, ADRs, code areas, and examples where useful.
5. When a decision changes, update the old ADR's status and add a new ADR explaining the replacement. Do not silently rewrite history.
6. Keep ADRs focused on durable decisions, not routine implementation notes or a chronological work log.

ADRs are part of the project's design documentation, not optional paperwork. For substantial design work, include the ADR in the same change as the implementation (or add a clearly marked Proposed ADR before implementation when the decision needs discussion).

## Validation

Before considering a change complete, run the relevant checks. For Rust changes, the standard project checks are:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

For changes that may affect CLI behavior or examples, also run the relevant commands, such as:

```sh
cargo run -- check examples/basic.mt
cargo run -- check examples/query.mt
cargo run -- run examples/query.mt
cargo run -- check examples/modules/main.mt
cargo run -- run examples/modules/main.mt
```

Do not claim a check passed unless it was actually run or CI confirms it. Report any checks that could not be run.

## Documentation map

- [Language Specification v0.1](docs/language-v0.1.md): current language behavior and syntax reference
- [MVP Scope](docs/mvp-scope.md): current MVP boundaries
- [Resolved Model](docs/resolved-model.md): resolved representation
- [Design Overview](docs/overview.md): historical design notes; defer to the language specification where they differ
- [ADRs](docs/adr/): durable design decisions
