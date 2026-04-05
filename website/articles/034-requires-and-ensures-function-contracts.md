---
title: Function Contracts With requires and ensures
date: 2026-04-05
author: Sigil Language Team
slug: requires-and-ensures-function-contracts
---

# Function Contracts With `requires` and `ensures`

Sigil already had `where` on named types, but that was always a type-level
surface. It defines which values belong to a type. It does not say who is at
fault when a function is called with the wrong assumptions, and it does not say
what a function promises back to its caller.

That distinction matters once the checker starts proving more facts across
function boundaries.

## The Problem

Type refinements are good for domain membership:

```sigil module
t BirthYear=Int where value>1800
```

But functions need a different vocabulary.

Sometimes the important fact is:

- the caller must establish a precondition before the call

Sometimes it is:

- the callee guarantees something stronger about the returned value

Trying to encode both of those with `where` would blur together three different
concepts:

- type membership
- caller obligations
- callee guarantees

Sigil prefers one canonical surface per concept, not one overloaded keyword
that means several different things depending on context.

## The Decision

Types keep `where`.

Functions now use `requires` and `ensures`:

```sigil module
λnormalizeYear(raw:Int)=>Int
requires raw>0
ensures result>1800
match raw>1800{
  true=>raw|
  false=>1900
}
```

The canonical runnable example for this surface now lives in
`language/examples/functionContracts.sigil`, and the companion measure/narrowing
example lives in `language/examples/proofMeasures.sigil`.

The split is intentional:

- `where` defines membership in a type
- `requires` defines what a caller must prove
- `ensures` defines what the callee guarantees after it returns

## Why Separate Keywords Are Better

`requires` and `ensures` are directional.

That is exactly what function contracts need. A function call has a before and
an after:

- before the call, the caller is responsible for the precondition
- after the call, the callee is responsible for the postcondition

Types are not directional in the same way. A type definition is simply saying
which values count as members of that type. `where` is still the right word for
that job because it is descriptive rather than temporal.

So Sigil keeps the language partition clean:

- type invariants use `where`
- function contracts use `requires` / `ensures`

## What The Compiler Checks

The compiler now checks contracts in two different places:

- `requires` is checked at call sites
- `ensures` is checked against the function body

If a call cannot prove the precondition, the call fails to typecheck. If a
function body cannot prove its own `ensures` clause, the function declaration
fails to typecheck.

That means blame is much clearer than in a single undifferentiated predicate
system:

- a failed `requires` clause points at the caller
- a failed `ensures` clause points at the function body

## Effectful Functions

Sigil also allows contracts on effectful functions, but the meaning stays
narrow on purpose.

Contracts may talk about:

- parameters
- the returned value (`result`)

They do not describe:

- effect history
- world transitions
- temporal behavior of IO

That keeps contracts in the same small, canonical proof language as type
refinements instead of turning them into a second effect-specification system.

## Why This Fits Sigil

This change makes the proof surface broader without making the language looser.

There is still one canonical way to express each kind of fact:

- use `where` for type membership
- use `requires` for preconditions
- use `ensures` for postconditions

The compiler can now prove more across call boundaries, and agents get a more
useful machine-readable failure surface, without forcing users to write proof
scripts or learn a second mini-language.

For how this moved from isolated snippets into real project code, see
`dogfooding-contracts-and-refinements`.
