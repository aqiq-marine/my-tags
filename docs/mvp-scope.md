# my-tags v0.1 MVP Scope

**Status:** Proposed scope agreed for v0.1  
**Date:** 2026-11-01

## Goal

Build the smallest usable implementation that can define an ontology, annotate whole videos with tags and properties, validate those annotations, and search them through a programmatic API.

The MVP is intended to validate the language's core data model. It is not intended to be a general-purpose knowledge representation or video-processing platform.

## In scope

### 1. Ontology

- Define named sets and membership/subset relationships using `in`.
- Reserve `Tag` and `Number` as the only primitive roots.
- Treat `Tag` as the root of concepts eligible to be used as tags. Membership in a property-value set does not by itself make a value a tag.
- Define properties with a domain and a range.
- Inherit properties through set inclusion.
- Allow a subtype to narrow an inherited property's range.
- Perform semantic validation: resolve names, check membership/type constraints, and detect invalid property definitions.

Example (illustrative syntax):

```text
Animal in Tag
Cat in Animal

Motion
HasDestination in Motion
walking in HasDestination

Animal has motion: Motion
Cat has motion: HasDestination
```

### 2. Data

- Annotate videos as whole objects; no temporal segments or timestamps in v0.1.
- Identify each video using a stable user-defined ID, not a filesystem path or URI as its identity.
- Store ontology and annotation data in files.
- Allow multiple tags and properties per video.
- Properties are optional: an annotation need not provide every property defined for its tag.
- Validate supplied tags and property values against the ontology.
- Reject unknown identifiers and values outside the declared property range.

Illustrative syntax:

```text
cat(
    motion: walking,
    color: white
): video_a
```

The concrete file format and module/import rules remain implementation decisions; the example is not yet a frozen grammar.

### 3. Query

- Expose search through a programmatic API.
- Do not implement a dedicated query DSL in v0.1.
- Support filtering by tag membership and property values.
- Return matching video IDs.
- Support boolean composition of conditions in the API where practical.
- Numeric comparisons may be supported for `Number`; ordering comparisons are not defined for ordinary set-valued properties.

The exact API shape and host language are implementation decisions.

### 4. Developer interface

- Provide a library/API for loading ontology and data files, validating them, and querying annotations.
- A small CLI for validation and diagnostics may be included as a development aid, but a CLI query language is not a v0.1 requirement.
- Report errors with the source file and location when available.

## Out of scope

- Time ranges, frame-level annotations, and timestamps.
- Relationships between distinct videos or other annotated entities.
- Image, audio, document, or arbitrary resource types as first-class targets.
- Query DSL syntax.
- Inference/reasoning beyond explicitly declared set inclusion and property inheritance.
- Union, intersection, and difference as ontology-definition operators.
- Multiple inheritance and complex property-constraint composition.
- Required-property constraints, defaults, and validation of absent optional properties.
- Dynamic ontology mutation and migration of existing data.
- Database-backed storage and external database integrations.
- GUI/editor, LSP, autocomplete, and automatic video analysis/tagging.

## Acceptance criteria

v0.1 is complete when an example project can:

1. Load ontology definitions from files.
2. Validate the ontology and report malformed or inconsistent definitions.
3. Load video annotations from files using stable video IDs.
4. Accept whole-video tags and optional properties.
5. Reject tags that are not eligible tags and property values outside their declared ranges.
6. Query annotations through a programmatic API using tag and property conditions.
7. Return the matching video IDs for a documented example query.

## Decisions intentionally left open

- Exact source grammar and file extensions.
- Module/import and multi-file organization.
- Exact distinction and type rules for sets, set members, and tag-eligible concepts.
- Exact API signatures and host programming language.
- Annotation identity and duplicate-annotation behavior.
- Representation of unknown or explicitly empty property values.
- Whether query evaluation is eager or can be backed by an index.

These decisions should be made only when needed by the implementation, while preserving the scope above.
