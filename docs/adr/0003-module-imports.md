# ADR 0003: Module imports and namespaces

- **Status:** Accepted
- **Date:** 2026-11-01

## Context

The language currently treats a parsed source file as one program. As the ontology grows, keeping every type, instance, and annotation in one file becomes difficult to maintain. Splitting files without a namespace policy would introduce collisions between common names such as `Atom`, `Sample`, or `Color`.

The language also needs predictable dependency behavior: repeated imports should not duplicate declarations, import aliases should not leak transitively, and cycles should be diagnosed rather than producing partially initialized symbol tables.

## Decision

Adopt file-based modules with explicit aliases and qualified external references.

### 1. One source file is one module

Every `.mt` file defines one module. The entry file is the root module. Imported files and their transitive dependencies are loaded as part of the same compilation unit.

### 2. Explicit imports

Use this syntax:

```text
import "chemistry.mt" as chemistry
import "../shared/units.mt" as units
```

- The path is resolved relative to the importing file.
- The alias is mandatory and must be a simple identifier.
- Imports are top-level declarations.
- Only local filesystem paths are supported in this version.

### 3. Explicit qualification for external symbols

A module may refer to its own declarations by local name. Declarations from an imported module require the import alias:

```text
import "chemistry.mt" as chemistry

def Sample {
  atom: chemistry::Atom
}
```

The `::` separator is used for module qualification. Property names remain local to their owning type. Declaration names and property names cannot be qualified.

### 4. Alias scope and visibility

An alias is visible only in the module containing the import declaration. Importing a module does not make that module's imports available to the importer. There is no implicit re-export.

All declarations in an imported module are currently visible through its alias. Explicit `pub`/private visibility is deferred until there is a concrete need.

### 5. Module and symbol identity

The canonical file path relative to the entry file's directory identifies a module within one compilation. A user-defined symbol is identified by the pair (module path, local name).

The current implementation lowers this pair to a canonical qualified symbol name before existing semantic analysis and resolution. The existing globally unique `TypeId` and `InstanceId` remain the runtime identities in the resolved model. A dedicated serialized `ModuleId` is not introduced in this version.

Built-in types (`Subject`, `Tag`, `Float`, `Int`, and `Nat`) remain globally available and are not qualified.

### 6. Dependency behavior

- Transitive imports are loaded recursively.
- A canonical file is parsed once per project load.
- Import aliases must be unique within a module.
- Importing the same canonical file twice from one module is rejected, even under different aliases.
- Cyclic imports are rejected, with the cycle chain included in the diagnostic.
- Import order does not affect symbol identity.
- Only `#show query` directives in the entry module are executed. Imported modules contribute declarations and statements.

## Consequences

### Positive

- Common local names can coexist in different modules.
- External dependencies are visible at each use site.
- Import aliases do not leak transitively.
- Repeated imports and cycles have deterministic behavior.
- Existing semantic validation and the ID-based resolved model can be reused after module-qualified names are lowered.

### Costs and limitations

- Users must qualify external symbols explicitly.
- Renaming an import alias requires updating references in that module.
- Symbol identity currently depends on the module's canonical path within the project.
- The first implementation does not provide explicit public/private declarations, wildcard imports, re-exports, remote imports, or package registries.
- Semantic diagnostics from imported modules currently use the existing single-source diagnostic interface; preserving each imported file's exact source excerpt and path is a follow-up improvement.

## Alternatives considered

### Implicitly merge all imported names into one namespace

Rejected because unrelated modules can define the same local name, and imports can silently change which declaration a name refers to.

### Require fully qualified paths instead of aliases

Rejected for the initial version because filesystem paths are verbose and path changes would leak into every reference. Import aliases provide a local, readable name.

### Implicit transitive re-export

Rejected because a module's public surface would change when its implementation dependencies change.

### Permit cyclic imports

Rejected for the initial version. The language currently has declarations and forward references, so acyclic module dependencies are sufficient and much easier to diagnose and reason about.

## References

- [Language Specification v0.1](../language-v0.1.md)
- [Go specification: package initialization](https://go.dev/ref/spec#Program_initialization)
- [Rust Reference: modules](https://doc.rust-lang.org/reference/items/modules.html)
- [OWL 2 Structural Specification: imports](https://www.w3.org/TR/owl2-syntax/#Imports)
