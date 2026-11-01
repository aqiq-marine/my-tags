# my-tags v0.1 MVP Scope

**Status:** Current implementation scope

## Goal

Build a small language implementation that defines types and instances, attaches tag instances to typed subjects through independent statements, validates the data, and supports searching through both a programmatic API and a small query DSL.

The core model is subject-agnostic. A video, chemical reaction, document, experiment, or other resource can be annotated when its type inherits from the built-in Subject type.

## Decisions

| Area | v0.1 decision |
|---|---|
| Annotation target | Any instance whose type inherits from Subject |
| Properties | Optional; each property has at most one value per statement |
| Storage | Source files |
| Query interface | Programmatic API and source-level query DSL (#show query) |
| Type inheritance | Single inheritance |
| Type composition | Intersection using + |
| Numeric types | Nat <: Int <: Float |
| Numeric widening | Implicit widening only; no narrowing |
| Float domain | Finite values only |
| Statements | Independent; repeated subject/tag pairs are allowed |
| Values | let defines named instances, not bare symbols |

## In scope

### Type definitions
- Define named types with def.
- Define property-free types and types with properties.
- Define a type with one parent type, optionally adding properties.
- Use intersection expressions such as Tag + Animal in instance type annotations.
- Inherit properties from a parent type.
- In v0.1, inherited properties are used as declared; overriding or narrowing inherited property ranges is not supported.
- Built-in types are Subject, Tag, Float, Int, and Nat; they cannot be redefined.
- Numeric types form the subtype chain Nat <: Int <: Float.

### Instances
- Define named instances with let.
- An instance may satisfy one type or an intersection of types.
- Values are instances, not untyped symbols.
- Names are unique within the relevant program/module scope.
- Nested anonymous instances are allowed in property values.

### Statements and annotations
- Use stmt <subject-id> has <tag-instance> to attach a tag to a subject.
- A statement subject must satisfy the built-in Subject type.
- A statement tag must satisfy the built-in Tag type.
- A statement may contain zero or more properties.
- Properties are optional and each may occur at most once within a single statement.
- Statements are independent. Multiple statements with the same subject and tag are retained separately.
- Statement properties belong to that statement, not to the tag instance itself.

### Numeric values
- Nat represents nonnegative integers, Int signed integers, and Float finite floating-point values.
- Nonnegative integer literals have type Nat; negative integer literals have type Int; decimal and exponent-form literals have type Float.
- Implicit widening is allowed from Nat to Int and from Int to Float.
- Narrowing conversions are rejected.
- NaN and positive or negative infinity are rejected.

### Validation
- Resolve type and instance names, including forward references.
- Check inheritance and type composition.
- Check that statement subjects satisfy Subject.
- Check that statement tags satisfy Tag.
- Check that statement properties are available on the tag's type.
- Check that property values satisfy the declared range, including subtype compatibility and numeric widening.
- Reject duplicate property names within one statement.
- Reject cyclic inheritance and duplicate declarations.

### Query
- Provide a programmatic API for filtering statements and subjects by tag and property values.
- Provide a source-level query DSL introduced by #show query.
- Query type blocks match statements whose tag has the requested type or a subtype.
- Property predicates include presence/absence, named-reference equality/inequality, and numeric comparisons.
- Support recursively nested typed anonymous-value patterns.
- Nested query patterns match anonymous values only; they do not traverse named references.
- Support boolean composition of predicates and set operations over matching subject instances.

### Storage and developer interface
- Ontology, instance, and statement data are stored in source files.
- A project may be split across multiple `.mt` modules using `import "path.mt" as alias`.
- Imports are resolved recursively relative to the importing file.
- External symbols are referenced with `alias::name`; aliases are module-local and are not re-exported.
- Duplicate aliases, repeated imports of the same file by one module, and cyclic imports are rejected.
- Provide a library/API to load, validate, resolve, and query those files.
- Provide CLI commands for source validation and query execution.
- Diagnostics should include source locations when available.

## Out of scope
- Time ranges, timestamps, and frame-level annotations.
- Relationships between distinct subjects or other entities.
- Multiple inheritance.
- Union types in type expressions (query set union remains in scope).
- Property overriding, range narrowing, and complex constraint composition.
- Required properties, defaults, and validation of absent optional properties.
- Dynamic ontology mutation and data migration.
- Database-backed storage and external database integrations.
- Remote imports, package registries, wildcard imports, re-exports, and configurable public/private visibility.
- GUI/editor, LSP, autocomplete, and automatic analysis/tagging.
- Inference beyond declared type inheritance and subtype compatibility.

## Acceptance criteria

v0.1 is complete when an example project can:
1. Load type and instance declarations from source.
2. Load statements connecting Subject instances to Tag instances.
3. Validate declarations, inheritance, tag eligibility, statement properties, and property value types.
4. Preserve multiple independent statements for the same subject/tag pair.
5. Preserve optional properties, with at most one value for each property in a statement.
6. Query through the programmatic API using tag and property conditions.
7. Parse, type-check, and execute a documented #show query example, including a nested anonymous-value pattern.
8. Return matching subject instance IDs for the documented example.
9. Load a project split across multiple modules, resolve qualified references, and reject import cycles and duplicate aliases.

## Related specification

See [Language Specification v0.1](language-v0.1.md) for the grammar and semantic rules.
