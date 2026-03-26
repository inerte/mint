---
title: Centralized Project Types and Constrained Type Meanings
date: 2026-03-26
author: Sigil Language Team
slug: centralized-project-types-and-constrained-type-meanings
---

# Centralized Project Types and Constrained Type Meanings

Sigil now treats project-defined types as part of the project's foundational
vocabulary instead of ordinary per-module implementation detail.

## The Problem

Once a project grows, domain types start to spread.

One module defines `User`, another defines `PersistedState`, a third adds
`Email`, and a fourth quietly invents a second near-duplicate wrapper for the
same concept. Even when the names are good, the vocabulary is fragmented.

That fragmentation creates two problems:

- the project has no single canonical place for its domain language
- many of the facts people care about end up back in comments because plain
  `Int` and `String` are too weak to carry the intended meaning

Sigil already prefers explicit, compiler-owned structure over conventions. Type
vocabulary should follow the same rule.

## The Decision

Projects now centralize named project types in one file:

```text
src/types.lib.sigil
```

That file is compiler-known and types-only. Outside it, project-defined types
are referenced through `µ...`.

Example:

```sigil module projects/algorithms/src/types.lib.sigil
t BirthYear=Int where value>1800 and value<10000

t TopologicalSortResult=CycleDetected()|Ordering([Int])

t User={birthYear:BirthYear,name:String}
```

```sigil module projects/algorithms/src/topologicalSortView.lib.sigil
λorderingValues(result:µTopologicalSortResult)=>[Int] match result{
  µOrdering(order)=>order|
  µCycleDetected()=>[]
}
```

This does two things at once:

- it gives the project one canonical home for named domain vocabulary
- it makes project-defined types visibly different from stdlib, config, world,
  and ordinary source-module references

## Constrained Types

Named user-defined types may also carry a pure `where` clause:

```sigil module projects/algorithms/src/types.lib.sigil
t BirthYear=Int where value>1800 and value<10000

t DateRange={end:Int,start:Int} where value.end≥value.start
```

The point is not to turn every type declaration into a runtime admission gate.
The point is to let types carry more semantic meaning directly in the source.

That means:

- `BirthYear` says more than bare `Int`
- `DateRange` says more than a record with two integers
- fewer invariants need to be repeated in comments

Current Sigil keeps this intentionally modest:

- `where` is pure and world-independent
- only `value` is in scope
- the compiler typechecks the constraint expression
- the checker rejects obvious literal contradictions
- there is no generated runtime validation and no solver-backed refinement
  system in this rollout

So this feature is about **stronger type meaning**, not about silently inserting
runtime checks.

## Type Equality

This also clarifies the structural-equality story.

Sigil still normalizes unconstrained aliases and unconstrained named product
types structurally. That part has not changed.

What changed is that constrained aliases and constrained
project-defined named products now stay distinct instead of normalizing all the
way down to their underlying shape. If a type carries extra semantic meaning,
the checker should not erase that distinction.

## Why This Matters

This is useful for both humans and tools.

Humans get one place to look for the project's domain vocabulary.

LLMs get a clearer, compiler-enforced project shape:

- `•...` for source modules
- `§...` for stdlib
- `†...` for world
- `※...` for test
- `µ...` for project-defined types

And the type declarations themselves can now carry some of the meaning that
would otherwise drift into comments.

That is the real goal of this change: not to make Sigil more ceremonial, but to
move more domain intent into checked, canonical source.
