# my-tags v0.1 MVP Scope

**Status:** Agreed MVP scope  
**Date:** 2026-11-01

## Goal

Build a small language implementation that defines types and instances, attaches tag instances to whole-video resources through independent statements, validates the data, and supports searching through a programmatic API.

The v0.1 goal is to validate the core language and data model, not to build a general-purpose knowledge representation system or video-processing platform.

## Decisions

| Area | v0.1 decision |
|---|---|
| Annotation granularity | Whole video only |
| Properties | Optional; each property has at most one value per statement |
| Storage | Source files |
| Query interface | Programmatic API; no query DSL |
| Video identity | Stable user-defined ID |
| Type inheritance | Single inheritance; inheritance with or without additional properties |
| Type composition | Intersection using `+` |
| Statements | Independent; repeated subject/tag pairs are allowed |
| Values | `let` defines named instances, not bare symbols |

## In scope

### Type definitions
- Define named types with `def`.
- Define property-free types and types with properties.
- Define a type with one parent type, optionally adding properties.
- Use intersection expressions such as `Tag + Animal` in instance type annotations.
- Inherit properties from a parent type.
- In v0.1, inherited properties are used as declared; overriding or narrowing inherited property ranges is not supported.

### Instances
- Define named instances with `let`.
- An instance may satisfy one type or an intersection of types.
- Values are instances, not untyped symbols.
- Names are unique within the relevant program/module scope.
- Nested anonymous instances are allowed in property values.

### Statements and annotations
- Use `stmt <video-id> has <tag-instance>` to attach a tag instance to a video.
- A statement may contain zero or more properties.
- Properties are optional.
- Each property may occur at most once within a single statement.
- Statements are independent. Multiple statements with the same video and tag are retained separately.
- Statement properties belong to that statement, not to the tag instance itself.
- Videos are identified by stable IDs.
- Only whole-video annotations are supported.

### Validation
- Resolve type and instance names, including forward references.
- Check inheritance and type composition.
- Check that statement subjects are video instances.
- Check that statement tags satisfy the special `Tag` type.
- Check that statement properties are available on the tag's type.
- Check that property values satisfy the declared range, including subtype compatibility.
- Reject duplicate property names within one statement.
- Reject cyclic inheritance and duplicate declarations.

### Query
- Provide a programmatic API for filtering statements/videos by tag and property values.
- Return matching video IDs.
- Boolean composition may be provided by the host API.
- No dedicated query language is included in v0.1.

### Storage and developer interface
- Ontology, instance, and statement data are stored in source files.
- Provide a library/API to load, validate, and query those files.
- A small CLI for validation and diagnostics is optional as a developer aid.
- Diagnostics should include source locations when available.

## Out of scope
- Time ranges, timestamps, and frame-level annotations.
- Relationships between distinct videos or other entities.
- Image, audio, document, or arbitrary resource types as first-class annotation targets.
- Query DSL.
- Multiple inheritance.
- Union types and general set algebra.
- Property overriding, range narrowing, and complex constraint composition.
- Required properties, defaults, and validation of absent optional properties.
- Dynamic ontology mutation and data migration.
- Database-backed storage and external database integrations.
- GUI/editor, LSP, autocomplete, and automatic video analysis/tagging.
- Inference beyond declared type inheritance and subtype compatibility.

## Acceptance criteria

v0.1 is complete when an example project can:
1. Load type and instance declarations from source files.
2. Load whole-video statements using stable video IDs.
3. Validate declarations, inheritance, tag eligibility, statement properties, and property value types.
4. Preserve multiple independent statements for the same video/tag pair.
5. Preserve optional properties, with at most one value for each property in a statement.
6. Query through a programmatic API using tag and property conditions.
7. Return matching video IDs for a documented example.

## Related specification

See [Language Specification v0.1](language-v0.1.md) for the provisional grammar and semantic rules.

## Intentionally deferred decisions
- Exact file extensions and module/import syntax.
- Exact API signatures and host programming language.
- Persistent identity format and cross-file namespace rules.
- Unknown/null values and explicit empty values.
- Whether query evaluation is eager or index-backed.

These details can be decided during implementation without expanding the agreed v0.1 scope.
