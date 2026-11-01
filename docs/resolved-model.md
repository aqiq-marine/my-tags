# Resolved Model

**Status:** Initial implementation design for v0.1  
**Source of truth:** .mt source files

The parser AST preserves source-level names and syntax. The Resolved Model is a separate ID-based representation produced only after semantic validation. Querying and optional serialized caches operate on this representation; they do not replace source files as the canonical data.

## Identity

- TypeId, InstanceId, and StatementId are distinct Rust newtypes.
- PropertyId is the pair (declaring TypeId, property index). An inherited property retains the identity of the type that declared it. Same-named properties on unrelated types remain distinct.
- IDs are deterministic for a given source declaration order, but are not promised to remain stable across source edits or reloads.

## Values

Named references resolve to InstanceId; numeric literals remain numbers; nested anonymous values carry their TypeId and recursively resolved properties. Anonymous values do not enter the named instance table.

## Statements

Every source statement receives a distinct StatementId, even when subject, tag, and properties match another statement. Statement order is preserved.

## Storage

The source file is canonical. JSON serialization is provided as a debugging/export format and can be regenerated from source. The initial JSON format is not a stable interchange format and does not promise ID stability between builds.

## Query API

The first query API is a Rust builder over statements:

```rust
let matching = model.query()
    .statements()
    .where_tag_is_subtype_of(animal_type)
    .collect();

let videos = model.query()
    .where_tag_is_subtype_of(animal_type)
    .subjects();
```

Statement queries preserve duplicates. subjects() explicitly returns distinct subjects in first-seen order.
