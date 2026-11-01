# Resolved Model

**Status:** Current implementation design for v0.1  
**Source of truth:** .mt source files

The parser AST preserves source-level names and syntax. The Resolved Model is a separate ID-based representation produced only after semantic validation. Querying and optional serialized caches operate on this representation; they do not replace source files as the canonical data.

## Identity

- TypeId, InstanceId, and StatementId are distinct Rust newtypes.
- PropertyId is the pair (declaring TypeId, property index). An inherited property retains the identity of the type that declared it. Same-named properties on unrelated types remain distinct.
- IDs are deterministic for a given source declaration order, but are not promised to remain stable across source edits or reloads.

## Built-in types

The model includes the built-in types Subject, Tag, Float, Int, and Nat. Int inherits from Float, and Nat inherits from Int. User-defined subject types inherit from Subject; statement subjects are validated against that root type.

## Values

Named references resolve to InstanceId. Numeric values are represented by NumericValue:
- Nat(u64) for nonnegative integer values.
- Int(i128) for signed integer values.
- Float(f64) for finite floating-point values.

Nonnegative integer literals are parsed as Nat, negative integer literals as Int, and decimal/exponent-form literals as Float. Numeric assignments are widened to the declared primitive property type when needed. The allowed implicit conversions are Nat to Int, Nat to Float, and Int to Float. Float values must be finite; NaN and infinities are not part of the language.

Nested anonymous values carry their TypeId and recursively resolved properties. Anonymous values do not enter the named instance table.

## Statements

Every source statement receives a distinct StatementId, even when subject, tag, and properties match another statement. Statement order is preserved. A statement subject must satisfy Subject, and its tag must satisfy Tag.

## Storage

The source file is canonical. JSON serialization is provided as a debugging/export format and can be regenerated from source. The initial JSON format is not a stable interchange format and does not promise ID stability between builds.

## Query API

Queries operate on statements and combine filters with AND. Each matching statement is retained independently, including repeated subject/tag pairs. subjects() explicitly returns distinct subjects in first-seen order.

~~~rust
let matching = model.query()
    .where_tag_is_subtype_of(animal_type)
    .where_property_exists(weight)
    .where_property_number_gt(weight, 10)
    .collect();
~~~

Numeric query methods accept integer and floating-point thresholds. Property types are checked by the language's numeric subtype rules. The initial implementation scans statements linearly; indexes are deferred.

## Query DSL

The source-level query DSL is separate from the programmatic query API. A #show query directive evaluates to a set of subject instances. A type-query block selects subjects for which at least one statement has a tag of the requested type (or a subtype) and satisfies all conditions in that block. Conditions in one block are evaluated against the same statement; separate blocks combine their subject sets with set operators.

Nested property values can be matched with typed anonymous-value patterns:

~~~text
#show query Hoge {
  huga: Piyo {
    bar: baz
  }
}
~~~

The pattern recursively inspects the anonymous value assigned to huga. The actual value may be an instance of Piyo or a subtype. A bare identifier such as baz is shorthand for reference equality. Nested patterns can contain further property predicates, including none and numeric comparisons.

These patterns do not follow named references: if huga contains a named Piyo reference rather than an anonymous value, the nested pattern does not match. The query DSL is parsed and type-checked by my-tags check, and evaluated in source order by my-tags run.
