# my-tags

A language and data model for describing videos with structured, ontology-aware tags.

The design separates three concerns:

1. **Ontology** — concepts, sets, membership, and properties.
2. **Data** — structured tag values attached to videos.
3. **Query** — searches over concepts and property constraints.

## Initial design direction

- `Tag` and `Number` are the only planned primitive roots.
- `Tag` is the root of elements eligible to be used as video tags. It does **not** imply that every leaf in the ontology is a tag.
- Other conceptual domains are built from named sets and `in` relationships.
- Properties are reusable named relations with declared value domains.
- Concepts may inherit and specialize properties.
- Missing properties are allowed initially; completeness checks belong to a linter.
- Ontology, data, and query files have separate responsibilities.
- The first version focuses on video-to-tag annotation. Relations between videos or arbitrary entities are out of scope.

These are design directions, not a finalized grammar or formal semantics. See [the design overview](docs/overview.md) and [ADR 0001](docs/adr/0001-initial-language-model.md).

## Possible project layout

```text
project/
├── ontology/
│   ├── core.tag
│   ├── animals.tag
│   └── motions.tag
├── data/
│   └── videos.data
└── queries/
    └── animals.query
```

File extensions and syntax are provisional.

## Status

This repository currently records the language design. The grammar, parser, evaluator, and storage model have not yet been specified.
