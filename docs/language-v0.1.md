# my-tags Language Specification v0.1

**Status:** Provisional; intended to guide the MVP implementation  
**Date:** 2026-11-01

This document records the initial grammar and semantics agreed for my-tags v0.1. Syntax details may be refined during parser implementation, but changes should preserve the MVP scope in [mvp-scope.md](mvp-scope.md).

## 1. Language model

The language has three declaration/usage layers:

1. **Type layer — `def`:** declares types and their properties.
2. **Instance layer — `let`:** declares named instances of types.
3. **Statement layer — `stmt`:** records an independent tag assertion about a video, with optional statement-local properties.

A tag instance and a statement attaching that tag to a video are distinct entities. Properties in a statement belong to that statement, not to the tag instance.

## 2. Type definitions

### 2.1 Property-free type

```text
def Appearance
def Color
def Video
```

### 2.2 Type with properties

```text
def Animal {
  appearance: Appearance
  weight: Number
}
```

A property declaration has a name and a value type (range). The enclosing type is the property's domain. `Video` is an ordinary user-defined type; declare it in the ontology before using it as a statement subject.

### 2.3 Inheritance without additional properties

```text
def Mammal: Animal
```

### 2.4 Inheritance with additional properties

```text
def Cat {
  favorite_color: Color
}: Mammal
```

v0.1 supports single inheritance. A child type inherits its parent's properties and may add new properties. It may not override an inherited property or narrow its range in v0.1.

## 3. Type expressions and Tag eligibility

The special type `Tag` marks instances eligible to be used as tags.

An instance may have an intersection type:

```text
let cat: Tag + Animal
```

This means the instance satisfies both `Tag` and `Animal`. An instance is eligible as a statement tag only if its type satisfies `Tag`. Merely being a property value, or belonging to an unrelated type, does not make an instance a tag.

v0.1 supports intersection with `+`; union types and general set algebra are out of scope. `Tag` and `Number` are built-in types and cannot be redefined. `Number` values are signed 64-bit integers (`i64`) and may be written as integer literals, including negative values. Decimal literals are not supported in v0.1.

## 4. Instance definitions

```text
let white: Color
let black: Color

let cat: Tag + Animal
let video_a: Video
```

- `let` declares a named instance, not a bare symbol.
- Different instances may have the same type.
- Names must be unique within the applicable program/module scope.
- Forward references are allowed; implementations should collect declarations before performing semantic validation.
- Nested anonymous instances are allowed as property values (see Section 5).

## 5. Statements

### 5.1 Basic form

```text
stmt video_a has cat
```

This records that the video instance `video_a` has the tag instance `cat`.

### 5.2 Statement with properties

```text
stmt video_a has cat {
  appearance: SolidColor {
    color: white
  }
}
```

The statement has:
- subject: `video_a`
- tag: `cat`
- statement-local property `appearance`
- a nested anonymous `SolidColor` instance whose `color` is `white`

### 5.3 Independent statements

```text
stmt video_a has cat {
  appearance: SolidColor {
    color: white
  }
}

stmt video_a has cat {
  appearance: SolidColor {
    color: black
  }
}
```

These are two distinct statements. They are not merged, and neither overwrites the other. Multiple statements with the same subject/tag pair are allowed.

### 5.4 Property cardinality and optionality

- A statement may omit all properties.
- Each property may occur at most once in one statement.
- A property may appear in multiple independent statements.
- Properties attached to a statement do not mutate the tag instance.

## 6. Property typing

Example type declarations:

```text
def Appearance
def Color

def SolidColor {
  color: Color
}: Appearance

def Animal {
  appearance: Appearance
}

let white: Color
let cat: Tag + Animal
let video_a: Video

stmt video_a has cat {
  appearance: SolidColor {
    color: white
  }
}
```

Typing rules:

1. A statement property must be declared by the tag's type or inherited from its parent type.
2. The value assigned to a property must satisfy the property's declared range.
3. A value whose type is a subtype of the declared range is valid.
4. A nested anonymous instance is checked using the same rules as a named instance.
5. Unknown properties and incompatible values are errors.
6. Missing properties are not errors in v0.1.

For example, `SolidColor` is valid for a property whose range is `Appearance`, because `SolidColor` inherits from `Appearance`.

## 7. Video subject and tag constraints

- A statement subject must be an instance satisfying the user-defined `Video` type (or a subtype of it).
- A statement tag must be an instance satisfying the special `Tag` type.
- The subject is identified by its stable user-defined instance ID.
- An annotation applies to the whole video. Time ranges and frame-level annotations are not part of v0.1.

## 8. Provisional EBNF

The following is an initial grammar sketch, not a complete lexer/parser specification.

```ebnf
program             = { declaration | statement | show_query } ;

declaration         = type_definition | instance_definition ;

type_definition     = "def" identifier
                      [ type_property_block ]
                      [ ":" type_expression ] ;

type_property_block = "{"
                      { identifier ":" type_expression }
                      "}" ;

type_expression     = type_term { "+" type_term } ;
type_term           = identifier ;

instance_definition = "let" identifier ":" type_expression ;

statement           = "stmt" identifier "has" identifier
                      [ statement_property_block ] ;

statement_property_block
                    = "{"
                      { identifier ":" value_expression }
                      "}" ;

value_expression    = identifier
                    | number_literal
                    | identifier "{" statement_property_block "}" ;

number_literal      = [ "-" ] digit { digit } ;

identifier          = letter { letter | digit | "_" } ;
```

The parser/AST distinguishes type property declarations from statement property assignments, even though both use brace-delimited syntax. The initial implementation supports line comments and numeric literals; strings and file/module handling remain future work.

## 9. Semantic validation

The v0.1 semantic analyzer should check at least:

- Duplicate type and instance declarations.
- Unknown type and instance names.
- Unknown parent types and cyclic inheritance.
- Invalid or duplicate property declarations.
- Validity of type intersections.
- Statement subjects are instances of the user-defined `Video` type.
- Statement tags satisfy `Tag`.
- Statement properties are declared on the tag type or inherited.
- Each statement property occurs no more than once.
- Property values satisfy the declared range, including subtype compatibility.

Forward references are permitted, so validation should not depend on declaration order.

## 10. AST direction

A minimal AST can use these top-level node kinds:

```text
Program
├── TypeDefinition
│   ├── name
│   ├── properties
│   └── parent_type?
├── InstanceDefinition
│   ├── name
│   └── type_expression
└── Statement
    ├── subject
    ├── tag
    └── properties
```

A recursive value representation is needed for nested anonymous instances:

```text
ValueExpression
├── Reference(name)
└── AnonymousInstance(type_name, properties)
```

## 11. Open decisions

The following are deliberately not fixed by this initial specification:

- File extensions and module/import syntax.
- Host programming language and exact query API signatures.
- Cross-file namespaces and identity rules.
- Unknown/null values and explicit empty values.
- Query indexing and optimization strategy.
- Exact diagnostics format.

These should be resolved as implementation requires, without silently expanding the v0.1 scope.

## 12. Queries

A query is an expression that evaluates to a set of statement subjects (instances). A query is introduced by a display directive:

```text
#show query Reaction {
  yield: (>= 80)
} & Hoge {
  huga: (== p1) || (== p2)
}
```

A type-query block selects subjects having at least one statement whose tag instance has the named type (or a subtype). Conditions inside one block are evaluated against the **same statement** and are implicitly ANDed. If a subject has multiple statements, it is selected when at least one matching statement satisfies every condition in that block. Separate type-query blocks are evaluated independently and their subject sets are combined by set operators.

A property named in a query must be declared on the queried type or one of its ancestors. An unknown property is a semantic error, not a query that silently returns an empty set.

### 12.1 Property predicates

```text
Reaction {
  yield: (>= 80) && (< 95)
  solvent: (== methanol) || (== ethanol)
  catalyst: none
}

Hoge {
  huga: Piyo {
    bar: baz
  }
}
```

A property predicate may contain a typed anonymous-value pattern. The example matches a statement whose `huga` value is an anonymous `Piyo` instance and whose nested `bar` property refers to the named instance `baz`. Nested patterns can be arbitrarily deep and are checked against the declared property ranges. The actual anonymous value may have the requested type or one of its subtypes.

In a property predicate, a bare identifier such as `solvent: methanol` or nested `bar: baz` is shorthand for reference equality (`(== methanol)` or `(== baz)`). All ordinary predicates, including `none` and numeric comparisons, are also available for nested properties. A pattern only matches anonymous values; it does not follow a named reference to another instance.

- Comparisons use parenthesized atoms: `(== value)`, `(!= value)`, `(> number)`, `(>= number)`, `(< number)`, and `(<= number)`.
- `&&` has higher precedence than `||`.
- Multiple property lines in a type-query block are ANDed.
- `none` matches a statement in which the property has no assigned value.
- A property declared on the type is allowed to be absent from an individual statement. `none` tests this statement-local absence; it does not mean that the property is undefined in the ontology.
- Numeric comparisons require a `Number` property. Reference equality/inequality requires a compatible named instance.

Property-level NOT is not part of v0.1. Use `none` for explicit absence.

### 12.2 Set operators

```text
#show query (Reaction {} & Hoge {}) - Asymmetric {}
#show query !(Reaction { yield: (>= 80) })
```

The operators are evaluated over sets of subject instances:

| Operator | Meaning |
|---|---|
| `!` | Complement |
| `&` | Intersection |
| `-` | Difference |
| `|` | Union |

Precedence, from highest to lowest, is unary `!`, then `&` and `-` (left associative), then `|`. Parentheses override precedence. The complement universe is the set of instances that occur as subjects of at least one statement in the current source model.

### 12.3 Query grammar

The implemented query grammar is summarized below:

```ebnf
show_query       = "#" "show" "query" query_expr ;
query_expr       = query_intersection { "|" query_intersection } ;
query_intersection
                 = query_unary { ("&" | "-") query_unary } ;
query_unary      = "!" query_unary | query_primary ;
query_primary    = type_query | "(" query_expr ")" ;
type_query       = identifier "{" { property_condition } "}" ;
property_condition
                 = identifier ":" property_predicate ;
property_predicate
                 = property_and { "||" property_and } ;
property_and     = property_atom { "&&" property_atom } ;
property_atom    = "none"
                 | "(" comparison value ")"
                 | identifier
                 | anonymous_pattern ;
anonymous_pattern
                 = identifier "{" { property_condition } "}" ;
comparison       = "==" | "!=" | ">" | ">=" | "<" | "<=" ;
```

Whitespace and line breaks separate adjacent property conditions; commas are optional separators. Comparisons are parenthesized so that the property predicate grammar remains unambiguous.

### 12.4 Executing display directives

`my-tags check file.mt` parses and type-checks queries without evaluating them. `my-tags run file.mt` evaluates each `#show query` directive in source order and prints the matching subject instance names.

The query AST is independent of the existing programmatic statement-query API. Query evaluation currently scans statements; indexing and optimization are future work.
