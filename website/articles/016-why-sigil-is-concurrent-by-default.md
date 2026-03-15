---
title: Why Sigil Is Concurrent by Default
date: 2026-03-03
author: Sigil Language Team
slug: why-sigil-is-concurrentByDefault
---

# Why Sigil Is Concurrent by Default

Sigil used to describe itself as "async by default," but that phrasing obscured
what the compiler was actually trying to do. The more precise statement is that
Sigil is concurrent by default.

## The Model

Sigil keeps one function form. User code does not introduce separate `async`
syntax, explicit futures, or manual `await` choreography.

Instead, the compiler starts independent work in source order, keeps values in a
promise-shaped form while they flow through expressions, and only joins them at
strict demand points.

## Why This Matters

The point is not to hide effects. Effects are still tracked in the type system.
The point is to avoid forcing the programmer to spell concurrency management at
every ordinary call site when the dependencies between operations are already
clear from the expression graph.

That gives Sigil a simpler surface while still allowing overlapping effectful
work where the program structure permits it.

## Why the Terminology Changed

"Async by default" suggested a looser runtime story than Sigil actually wanted.
"Concurrent by default" is narrower and more accurate: the compiler preserves the
declaration of effects, but it does not serialize independent work unless the
program demands it.
