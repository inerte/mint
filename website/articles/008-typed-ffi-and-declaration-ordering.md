---
title: Typed FFI and the Declaration Ordering Change
date: 2026-02-25
author: Sigil Language Team
slug: typed-ffi-and-declaration-ordering
---

# Typed FFI and the Declaration Ordering Change

This article records the point where Sigil's early declaration-ordering rule had
to change to support typed FFI declarations. Some syntax and implementation
details have changed since then, but the dependency argument behind the ordering
change is still relevant.

## The Problem

Untyped FFI boundaries are easy to add, but they weaken one of Sigil's main
goals: keeping important program structure visible to the compiler. Once FFI
declarations started carrying real type information, they needed to be able to
reference named Sigil types.

That created a simple dependency issue. If a file required extern declarations
to appear before type declarations, then typed externs would have to either:

- avoid named types
- use forward-reference tricks
- or force the compiler to special-case the ordering rule

None of those were good outcomes.

## The Decision

Sigil changed the canonical declaration order so that types come first. That let
typed FFI declarations depend on named types in the same direct way that other
typed declarations do.

The exact historical ordering changed from an earlier `e => i => t ...` shape to
`t => e => c => λ => test`.

## Why the Change Was Worth It

The important part was not merely accommodating typed FFI. It was keeping the
ordering rule dependency-aware instead of arbitrary. If types are a prerequisite
for typed externs, then the canonical source order should reflect that fact.

This is one of the recurring patterns in Sigil's canonicality design: when a
deterministic rule and the language's real dependency structure diverge, the rule
should be updated rather than preserved for inertia.

## Consequence

Typed FFI became a cleaner part of the language, and canonical declaration
ordering stayed aligned with the dependencies expressed in the source rather
than with a historical layout decision.
