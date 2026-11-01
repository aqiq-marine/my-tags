# my-tags Language Specification v0.1

**Status:** Provisional; intended to guide the MVP implementation  
**Date:** 2026-11-01

This document records the initial grammar and semantics agreed for my-tags v0.1. Syntax details may be refined during parser implementation, but changes should preserve the MVP scope in [mvp-scope.md](mvp-scope.md).

## 1. Language model

The language has three declaration/usage layers:

1. **Type layer — `def`:** declares types and their properties.
2. **Instance layer — `let`:** declares named instances of types.
3. **Statement layer — `stmt`:** records an independent tag assertion about a subject, with optional statement-local properties.

A tag instance and a statement attaching that tag to a subject are distinct entities. Properties in a statement belong to that statement, not to the tag instance. A subject may be any instance whose type inherits from the built-in Subject type.

## 2. Type definitions

### 2.1 Property-free type

```text
def Appearance
def Color
def Video: Subject
```

### 2.2 Type with properties

```text
def Animal {
  appearance: Appearance
  weight: Float
}
```

A property declaration has a name and a value type (range). The enclosing type is the property's domain. Application-specific subject types, such as Video or ChemicalReaction, inherit from Subject.

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

The built-in type `Subject` is the root type for statement subjects. The special type `Tag` marks instances eligible to be used as tags.

An instance may have an intersection type:

```text
let cat: Tag + Animal
```

This means the instance satisfies both `Tag` and `Animal`. An instance is eligible as a statement tag only if its type satisfies `Tag`. Merely being a property value, or belonging to an unrelated type, does not make an instance a tag.

v0.1 supports intersection with `+`; union types in type expressions are out of scope. The built-in types `Subject`, `Tag`, `Float`, `Int`, and `Nat` cannot be redefined. Numeric types form the subtype chain `Nat <: Int <: Float`. `Nat` values are nonnegative integers (`u64`), `Int` values are signed integers (`i128`), and `Float` values are finite floating-point numbers (`f64`). Implicit widening is allowed from Nat to Int and from Int to Float; narrowing is rejected. Nonnegative integer literals are Nat, negative integer literals are Int, and decimal or exponent-form literals are Float. NaN and positive or negative infinity are rejected.

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

This records that the subject instance `video_a` has the tag instance `cat`.

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

## 7. Subject and tag constraints

- A statement subject must be an instance satisfying the built-in `Subject` type (or a subtype of it).
- A statement tag must be an instance satisfying the special `Tag` type.
- The subject is identified by its user-defined instance ID.
- A statement annotates its subject as a whole. Time ranges and frame-level annotations are not part of v0.1.

## 8. Provisional EBNF

The following is an initial grammar sketch, not a complete lexer/parser specification.

```ebnf
program             = { import_decl } { declaration | statement | show_query } ;

import_decl         = "import" string "as" identifier ;

declaration         = type_definition | instance_definition ;

type_definition     = "def" identifier
                      [ type_property_block ]
                      [ ":" type_expression ] ;

type_property_block = "{"
                      { identifier ":" type_expression }
                      "}" ;

type_expression     = type_term { "+" type_term } ;
type_term           = qualified_name ;

instance_definition = "let" identifier ":" type_expression ;

qualified_name      = identifier { "::" identifier } ;

statement           = "stmt" qualified_name "has" qualified_name
                      [ statement_property_block ] ;

statement_property_block
                    = "{"
                      { identifier ":" value_expression }
                      "}" ;

value_expression    = qualified_name
                    | numeric_literal
                    | qualified_name "{" statement_property_block "}" ;

string              = '"' { character } '"' ;
numeric_literal     = [ "-" ] ( integer_literal | float_literal ) ;
integer_literal     = digit { digit } ;
float_literal       = digit { digit } "." digit { digit } [ exponent ]
                    | "." digit { digit } [ exponent ]
                    | digit { digit } exponent ;
exponent            = ( "e" | "E" ) [ "+" | "-" ] digit { digit } ;

identifier          = letter { letter | digit | "_" } ;
```

The parser/AST distinguishes type property declarations from statement property assignments, even though both use brace-delimited syntax. The implementation supports line comments, numeric literals, quoted import paths, and module-qualified names. See Section 10 for multi-file module semantics.

## 9. Module imports and namespaces

### 9.1 Import declarations

A source file is one module. Imports are top-level declarations and use an explicit alias:

\`\`\`text
import "chemistry.mt" as chemistry
import "biology.mt" as biology
\`\`\`

The path is resolved relative to the importing file. Paths are local filesystem paths; URL and package-registry imports are not supported. The alias must be a simple identifier and is visible only in the importing module. An import does not re-export the imported module or its aliases.

### 9.2 Qualified names

Definitions are local to their declaring module. A module may refer to its own types and instances by unqualified names. A symbol from an imported module must be qualified with the import alias and the `::` separator:

\`\`\`text
import "chemistry.mt" as chemistry

def Sample {
  atom: chemistry::Atom
}

let sample: Sample
stmt video has chemistry::sample_atom
\`\`\`

Property names remain local to the declaring type and are not module-qualified. Declaration names and property names must be unqualified. Import aliases are not implicitly available through transitive imports.

### 9.3 Loading and identity

The entry file and its transitive imports are loaded as one compilation unit. Each canonical file path is loaded once. A file imported more than once under different aliases by the same importer is rejected; duplicate aliases are rejected. Cyclic imports are rejected and the diagnostic reports the dependency chain.

The canonical module path relative to the entry file directory, together with the local symbol name, identifies a user-defined type or instance. Thus `chemistry.mt::Atom` and `biology.mt::Atom` are distinct symbols even when their local names match. Built-in types (`Subject`, `Tag`, `Float`, `Int`, and `Nat`) remain globally available and are not qualified.

Only `#show query` directives in the entry module are executed. Imported modules contribute declarations and statements, not entry-point queries. All definitions in an imported module are currently visible to importers; explicit `pub`/private visibility is deferred.

### 9.4 Import restrictions

- Imports must be top-level declarations.
- Import aliases must be unique within a module.
- The same canonical file may not be imported twice by one module.
- Cyclic imports are errors.
- Wildcard imports and re-exports are not supported.
- Only local filesystem paths are supported.

## 10. Semantic validation

The v0.1 semantic analyzer should check at least:

- Duplicate type and instance declarations.
- Unknown type and instance names.
- Unknown parent types and cyclic inheritance.
- Invalid or duplicate property declarations.
- Validity of type intersections.
- Statement subjects satisfy the built-in `Subject` type.
- Statement tags satisfy `Tag`.
- Statement properties are declared on the tag type or inherited.
- Each statement property occurs no more than once.
- Property values satisfy the declared range, including subtype compatibility and numeric widening.

Forward references are permitted, so validation should not depend on declaration order.

## 11. AST direction

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

## 12. Open decisions

The following are deliberately not fixed by this initial specification:

- Host programming language and exact query API signatures.
- Unknown/null values and explicit empty values.
- Query indexing and optimization strategy.
- Exact diagnostics format.

These should be resolved as implementation requires, without silently expanding the v0.1 scope.

## 13. Queries

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

### 13.1 Property predicates

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
- Numeric comparisons require a numeric property (`Nat`, `Int`, `Float`, or a subtype). Numeric literals must be compatible with the property's numeric type. Reference equality/inequality requires a compatible named instance.

Property-level NOT is not part of v0.1. Use `none` for explicit absence.

### 13.2 Set operators

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

### 13.3 Query grammar

The implemented query grammar is summarized below:

```ebnf
show_query       = "#" "show" "query" query_expr ;
query_expr       = query_intersection { "|" query_intersection } ;
query_intersection
                 = query_unary { ("&" | "-") query_unary } ;
query_unary      = "!" query_unary | query_primary ;
query_primary    = type_query | "(" query_expr ")" ;
type_query       = qualified_name "{" { property_condition } "}" ;
property_condition
                 = identifier ":" property_predicate ;
property_predicate
                 = property_and { "||" property_and } ;
property_and     = property_atom { "&&" property_atom } ;
property_atom    = "none"
                 | "(" comparison value ")"
                 | qualified_name
                 | anonymous_pattern ;
anonymous_pattern
                 = qualified_name "{" { property_condition } "}" ;
comparison       = "==" | "!=" | ">" | ">=" | "<" | "<=" ;
```

Whitespace and line breaks separate adjacent property conditions; commas are optional separators. Comparisons are parenthesized so that the property predicate grammar remains unambiguous.

### 13.4 Executing display directives

`my-tags check file.mt` parses and type-checks queries without evaluating them. `my-tags run file.mt` evaluates each `#show query` directive in source order and prints the matching subject instance names.

The query AST is independent of the existing programmatic statement-query API. Query evaluation currently scans statements; indexing and optimization are future work.
