# Primitive Type Spelling

## Question

If Sigil moves away from Unicode primitive type glyphs, should primitive types use:

- lowercase ASCII: `int`, `float`, `bool`, `string`, `char`, `unit`, `never`
- capitalized ASCII: `Int`, `Float`, `Bool`, `String`, `Char`, `Unit`, `Never`

## Current findings

### Token cost

Using the repo tokenizer harness:

- lowercase and capitalized ASCII primitive names are tied on token cost
- this is true in standalone form and in simple typed contexts like `x:int` vs `x:Int`
- the meaningful token-cost win is ASCII over Unicode, not lowercase over capitalized

Representative results:

| Pair | Standalone | Typed context |
|---|---|---|
| `int` vs `Int` | tied across all local tokenizers | tied across all local tokenizers |
| `float` vs `Float` | tied across all local tokenizers | tied across all local tokenizers |
| `bool` vs `Bool` | tied across all local tokenizers | tied across all local tokenizers |
| `string` vs `String` | tied across all local tokenizers | tied across all local tokenizers |
| `char` vs `Char` | tied across all local tokenizers | tied across all local tokenizers |
| `unit` vs `Unit` | tied across all local tokenizers | tied across all local tokenizers |
| `never` vs `Never` | tied across all local tokenizers | tied across all local tokenizers |

Notes:

- `openai_cl100k_base` is exact in the local benchmark harness
- the Llama and Anthropic numbers are local heuristic proxies, so they are directional only

### Language-design tradeoff

Once Unicode is removed, this becomes a canonicality and readability decision rather than a tokenizer decision.

Lowercase benefits:

- matches the rest of Sigil's keyword-heavy surface
- visually distinguishes built-in primitives from user-defined types
- aligns with many languages where primitive/builtin types are keyword-like
- avoids making primitives look nominal

Capitalized benefits:

- makes all types visually uniform
- reinforces "types look like types" in prose and examples
- aligns with the internal AST names already used by the compiler (`PrimitiveName::Int`, `Bool`, etc.)

## Recommendation

Prefer lowercase ASCII primitive names:

`int`, `float`, `bool`, `string`, `char`, `unit`, `never`

Reasoning:

1. Token cost does not justify capitalized forms.
2. Lowercase better communicates that these are built-in language primitives, not user-defined nominal types.
3. It preserves a useful visual distinction between:
   - built-in primitive vocabulary
   - user-defined types like `User`, `Todo`, `Result`
4. It keeps Sigil's surface more keyword-like and regular.

If the project instead decides that "all types should look alike" is the stronger aesthetic rule, then capitalized forms are acceptable from a tokenizer perspective. This is a style choice, not a cost choice.

## Migration scope

This rename is mechanically straightforward but broad.

Expected touchpoints:

- lexer token definitions for primitive type spellings
- parser primitive-type matching
- typechecker error formatting
- syntax docs and spec examples
- stdlib signatures
- examples and projects
- test fixtures and parser/typechecker tests
- benchmarks that currently assume Unicode primitive spellings

Current codebase indicators:

- lexer currently hard-codes Unicode primitive tokens
- parser maps those tokens to `PrimitiveName::{Int,Float,Bool,String,Char,Unit}`
- typechecker diagnostics currently render Unicode names
- there are more than 1500 Unicode primitive occurrences outside compiler internals that would need source updates

## Suggested implementation order

1. Decide the canonical ASCII spellings.
2. Update lexer tokens and parser acceptance.
3. Update diagnostic rendering.
4. Update docs/spec/reference material.
5. Rewrite stdlib, examples, fixtures, and projects.
6. Re-run benchmark and compiler tests.

## Non-goal

Do not support both Unicode and ASCII primitive spellings long-term.

That would directly weaken Sigil's canonical-syntax goal.
