# Canonical Forms in Sigil

Sigil enforces canonical forms so one valid program has one accepted surface.

This document records the current canonical rules enforced by the lexer,
parser, validator, and typechecker in this repository.

## Why Canonical Forms Exist

Canonical forms are not style guidance. They are part of the language contract.

Goals:

- remove alternative spellings for the same construct
- improve deterministic code generation
- make diagnostics corrective instead of advisory
- keep examples, tests, and generated code aligned

## File Purpose

Sigil uses file extensions to distinguish file purpose:

- `.lib.sigil` for libraries
- `.sigil` for executables and tests

Current canonical rules include:

- `.lib.sigil` files must not define `main`
- non-test `.sigil` files must define `main`
- `test` declarations are only allowed under `tests/`

## Filename Rules

Basenames must be `lowerCamelCase`.

Valid:

- `hello.sigil`
- `userService.lib.sigil`
- `example01Introduction.sigil`

Invalid:

- `UserService.sigil`
- `user_service.lib.sigil`
- `user-service.sigil`
- `1intro.sigil`

Current filename diagnostics:

- `SIGIL-CANON-FILENAME-CASE`
- `SIGIL-CANON-FILENAME-INVALID-CHAR`
- `SIGIL-CANON-FILENAME-FORMAT`

## Declaration Ordering

Top-level declarations must appear in this category order:

```text
t → e → i → c → λ → test
```

Module scope is declaration-only.
Top-level `l` is invalid.

## No `export` Keyword

Current Sigil does not have an `export` token.

Visibility is file-based:

- declarations in `.lib.sigil` files are importable
- `.sigil` files are executable-oriented

## Function and Lambda Surface

Canonical function/lambda rules:

- parameter types are required
- return types are required
- effects, when present, appear between `→` and the return type
- `=` is required before non-`match` bodies
- `=` is forbidden before `match` bodies

Examples:

```sigil
λadd(x:Int,y:Int)→Int=x+y

λfactorial(n:Int)→Int match n{
  0→1|
  1→1|
  value→value*factorial(value-1)
}

λ(x:Int)→Int=x*2
```

## Constants

Current constant syntax is typed value ascription:

```sigil
c answer=(42:Int)
```

The older `c answer:Int=42` form is not current Sigil.

## Records and Maps

Records and maps are distinct.

- records use `:`
- maps use `↦`

Examples:

```sigil
t User={id:Int,name:String}
t Scores={String↦Int}
```

Record fields are canonical alphabetical order in:

- product type declarations
- record literals
- typed record constructors
- record patterns

## Local Binding Rules

Local names must not shadow names from the same or any enclosing lexical scope.

This applies to:

- function parameters
- lambda parameters
- `l` bindings
- pattern bindings

## Single-Use Pure Bindings

Sigil currently rejects pure local bindings used exactly once.

Example:

```sigil
λformulaText(checksums:Checksums,version:String)→String={
  l repo=(releaseRepo():String);
  src⋅formula.formula({checksums:checksums,repo:repo,version:version})
}
```

Required canonical form:

```sigil
λformulaText(checksums:Checksums,version:String)→String=
  src⋅formula.formula({checksums:checksums,repo:releaseRepo(),version:version})
```

Current mechanical rule:

- if a local binding is pure
- and the bound name is used exactly once
- the binding is rejected and must be inlined

The current validator does not perform a separate “substitution legality”
analysis. This document describes the implementation as it exists today.

## Topology / Config Boundaries

For topology-aware projects:

- topology declarations live in `src/topology.lib.sigil`
- selected environment bindings live in `config/<env>.lib.sigil`
- `process.env` is only allowed in `config/*.lib.sigil`
- application code must use topology dependency handles, not raw endpoints

Validation is currently per selected `--env`, not a whole-project scan across
all declared environments.

## Formatting Rules

Current canonical formatting rules include:

- file must end with a final newline
- no trailing whitespace
- at most one consecutive blank line

The lexer also rejects:

- tab characters
- standalone `\r`

## Validation Pipeline

Canonical validation happens in two stages:

1. after parsing, for syntax- and structure-level canonical rules
2. after typechecking, for typed canonical rules such as single-use pure
   bindings

The overall pipeline is:

```text
read source
→ tokenize
→ parse
→ canonical validation
→ typecheck
→ typed canonical validation
→ codegen / run / test
```

## Source of Truth

When prose disagrees with implementation, current truth comes from:

- parser
- validator
- typechecker
- runnable examples and tests
