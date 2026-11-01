> **Status:** Historical design notes. The initial `in`-based notation described below has been superseded by the `def` / `let` / `stmt` language model in [Language Specification v0.1](language-v0.1.md). Use that specification as the implementation reference.

# Language design overview

This document records the current design direction. Examples are illustrative and do not define a finalized grammar.

## 1. Goals

The language should support:

- Hierarchical concepts that can be queried through inclusion.
- Structured tags with named properties.
- Property value constraints expressed through concepts or sets.
- Reuse of ontology knowledge when querying broader concepts.
- Separation between concept definitions, video annotations, and searches.

The initial scope is intentionally narrow: describe video properties by attaching tag values to videos. Relations between videos or arbitrary entities can be considered later.

## 2. Sets, elements, and `in`

The model uses named sets and membership/inclusion relationships. The intended meaning of `in` depends on the kinds of its operands:

- An element may belong to a set.
- A set may be included in another set.

Both use the same surface relation, with semantic checks distinguishing the cases.

```text
Motion
HasDestination in Motion
walking in HasDestination
running in HasDestination
```

Here, `Motion` and `HasDestination` are sets, while `walking` and `running` are elements of `HasDestination`. This hierarchy does not imply that every leaf is a tag.

The language should reject invalid uses based on whether a name denotes a set or an element. The exact inference rules remain to be formalized.

## 3. Primitive roots: `Tag` and `Number`

Only `Tag` and `Number` receive primitive treatment initially.

- `Tag` is the root set for values eligible to be used as tags attached to a video.
- `Number` is the numeric domain, with numeric literals and numeric operations.

**Not every leaf in the ontology is necessarily a member of `Tag`.** A leaf such as `walking` can be a valid value in a property domain without being independently attachable to a video as a tag.

For example:

```text
Animal in Tag
Cat in Animal

Motion
HasDestination in Motion
walking in HasDestination
```

In this sketch, `Animal` is eligible as a tag concept because it is under `Tag`; `walking` is a value in a property-oriented hierarchy and is not thereby made a tag. The distinction between a tag concept, a tag value, and a set needs a precise type-system definition.

## 4. Properties

A property is a named semantic relation, not merely a function parameter local to one constructor. A property name should be reusable across unrelated concepts.

```text
Animal has motion: Motion
Animal has color: Color
Animal has size: Number
```

This declares that an Animal may have `motion`, `color`, and `size` properties, with the corresponding value domains.

A property need not be specific to animals. For example, vehicles or other concepts may also have motion.

### Inheritance and specialization

A subconcept may inherit properties from a broader concept and narrow a property's value domain.

```text
Animal has motion: Motion
Cat in Animal
Cat has motion: HasDestination
```

This is intended to mean that Cat inherits `motion` but specializes its allowed values. Whether specialization must always be a subset, and how multiple inheritance is handled, needs formal rules.

### Missing properties

The current preference is to allow a property to be absent from an annotation, even if the ontology declares it. Missing values should not automatically make an annotation ill-typed.

A linter may report missing information when an application expects a property to be present. This separates ontology-level validity from data completeness.

## 5. Structured tag values

A tag attached to a video can carry property values:

```text
cat(
    motion: walking,
    color: white,
    size: 30
): video_a
```

The intended reading is that a Cat tag value, with the specified properties, is attached to `video_a`.

This differs from attaching `walking` directly as a tag. Here, `walking` is the value of the `motion` property.

The exact construction syntax, identity of repeated tag values, handling of multiple instances of the same concept, and representation of unknown values are not settled.

## 6. Three language/file roles

The design separates three concerns. They may share lexical and syntactic conventions, but each file role has its own allowed declarations.

### Ontology

Defines the vocabulary and its semantics:

- Named sets and concepts.
- Membership and subset relationships through `in`.
- Property declarations and value domains.
- Inheritance and specialization.

Possible layout:

```text
ontology/
├── core.tag
├── animals.tag
├── motions.tag
└── colors.tag
```

### Data

Defines videos and their annotations:

- Video identifiers or references.
- Tag values attached to videos.
- Property values on those tag values.

Possible layout:

```text
data/
└── videos.data
```

### Query

Describes constraints used to retrieve matching videos:

- General concepts, such as `Animal`.
- Property presence, such as requiring a `motion` property.
- Property value constraints, such as `motion: walking`.
- Set operations and combinations of constraints.

Illustrative queries:

```text
Animal
Animal(motion: walking)
Animal(motion: Motion)
```

A query for a broader concept should match annotations whose concepts are included in it. For example, an annotation of `Cat` should be found by an `Animal` query when `Cat in Animal` is declared.

A query that asks for a property by name without specifying a particular value should also be possible. The exact syntax for property-presence queries is open.

## 7. Set operations

Set operations are desirable because they allow users to compose concepts and constraints without introducing many special-purpose operators.

Candidate operations:

- Union.
- Intersection.
- Difference.

They should work consistently in ontology expressions and queries where meaningful. Typing rules for combining sets, elements, property domains, and tag-capable concepts must be specified before implementation.

## 8. Initial scope boundaries

The first version intentionally does not model:

- Relations between two videos.
- Relations between arbitrary annotated entities.
- Temporal segments or spatial regions within a video.
- Full logical inference or a general-purpose ontology reasoner.
- Mandatory properties or closed-world validation.

## 9. Open design questions

1. What is the exact distinction between a set, an element, a tag-capable concept, and a constructed tag value?
2. Is `Tag` a set containing eligible concepts, or a separate root type with additional construction rules?
3. What are the precise inference rules for set-in-set and element-in-set relationships?
4. Can an element be used as a property value, and can a set be used as a property value constraint?
5. How are property declarations inherited through multiple paths?
6. Must property specialization be a subset of the inherited domain?
7. How should property presence be expressed in queries?
8. What are the syntax and typing rules for set operations?
9. How are duplicate annotations and multiple instances of the same concept represented?
10. Which checks belong to static type checking, and which belong to a linter?

These questions should be resolved incrementally as representative examples and implementation experience accumulate.
