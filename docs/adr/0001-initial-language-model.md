# ADR 0001: Initial language model and scope

- **Status:** Superseded by [Language Specification v0.1](../language-v0.1.md)
- **Date:** 2026-11-01

## Context

The project needs a model more expressive than flat video labels. A video may be annotated with a concept and structured property values, and users should be able to search through broader concepts and property constraints.

A plain bipartite graph between videos and labels is a useful baseline, but does not by itself express concept hierarchies, reusable properties, or structured tag values. The language therefore needs an ontology layer in addition to annotation data.

The first version should not become a general-purpose knowledge representation language. Relations between arbitrary entities are not required for the initial use case.

## Decision

Adopt the following as the initial design direction.

### 1. Separate ontology, data, and query

Use three logical file roles:

- **Ontology:** sets/concepts, membership, properties, and value domains.
- **Data:** structured tag values attached to videos.
- **Query:** constraints for retrieving videos.

The roles may share syntax, but declarations should be restricted according to each file's purpose.

### 2. Use `Tag` and `Number` as primitive roots

Only `Tag` and `Number` receive primitive treatment initially.

`Tag` is the root for concepts/values eligible to be used as video tags. It does not mean every leaf in every set hierarchy is a tag. For example, `walking` may be a member of a property domain without being independently attachable to a video as a tag.

`Number` represents numeric values and supports numeric literals and operations.

Other conceptual domains should be expressible through named sets and relationships.

### 3. Use `in` for membership and inclusion

Use a common `in` relation for element-in-set and set-in-set relationships. Static analysis must distinguish the cases and reject invalid combinations.

Detailed inference and typing rules are deferred until representative examples have been collected.

### 4. Model properties as reusable named relations

A property is a semantic relation, not merely a local function argument. A concept may declare a property and its value domain:

```text
Animal has motion: Motion
Animal has color: Color
```

Subconcepts may inherit and specialize property domains. The expected direction is narrowing to a subset of the inherited domain, subject to formalization.

### 5. Permit incomplete annotations

A declared property need not be present in every annotation. Missing values are allowed by default. Applications may use a linter to report missing or underspecified properties.

### 6. Represent annotations as structured tag values

A video annotation may contain a tag concept together with named property values:

```text
cat(
    motion: walking,
    color: white,
    size: 30
): video_a
```

The exact grammar and instance identity rules remain open.

### 7. Keep the initial scope video-centric

The initial data model supports attaching tag values to videos. It does not yet support arbitrary relations between videos or arbitrary entities.

### 8. Support set operations as a design goal

Union, intersection, and difference are desirable for composing ontology expressions and queries. Their exact syntax and typing rules are deferred.

## Consequences

### Positive

- The ontology can express more than a flat tag vocabulary.
- General concepts can retrieve annotations of more specific concepts.
- Property names and value domains can be reused across different concepts.
- Separating ontology, data, and query keeps responsibilities understandable.
- Allowing incomplete annotations avoids forcing exhaustive metadata on every video.

### Costs and risks

- The meaning of `in` requires a clear type/inference model because it covers membership and inclusion.
- The distinction between tag-capable concepts and ordinary property values must be enforced.
- Property inheritance and specialization need precise rules, especially with multiple inheritance.
- Query semantics must define whether a constraint means exact equality, set membership, or subset compatibility.
- A linter and a type checker need clearly separated responsibilities.

## Alternatives considered

### Flat labels only

A video is connected to a collection of independent tags.

Rejected as the sole model because it cannot express structured properties and reusable concept hierarchies.

### General graph of arbitrary entities and relations

Every entity can be connected to every other entity through arbitrary relations.

Deferred. It may eventually be useful, but it adds scope before the basic video annotation model is established.

### Treat every leaf as a tag

Rejected as the default interpretation. Some leaves are intended only as values within a property's domain, such as `walking` under a motion-related set. Tag eligibility should be represented explicitly through the `Tag` root.

## Unresolved questions

- Formal semantics of sets, elements, and tag construction.
- Exact type rules for `in`.
- Property inheritance and specialization rules.
- Query syntax for property-presence constraints.
- Set-operation syntax and typing.
- Annotation identity, multiplicity, and unknown values.
- Module/import behavior across ontology files.

## References

- [Language design overview](../overview.md)
