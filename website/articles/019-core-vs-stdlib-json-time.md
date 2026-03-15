---
title: Core vs Stdlib Is About Canonical Ownership, Not Purity
date: 2026-03-04
author: Sigil Language Team
slug: core-vs-stdlib-json-time
---

# Core vs Stdlib Is About Canonical Ownership, Not Purity

`Map` lives in Sigil's core collection surface. `json` and `time` live in the
standard library. That split is not a purity claim. It is an ownership claim.

## The Rule

Core is for concepts that are foundational enough to shape the language's common
vocabulary. Stdlib is for operational domains that should still have one
canonical home, but do not need to be implicit language-level concepts.

Under that rule:

- `Map` belongs in core
- `json` belongs in stdlib
- `time` belongs in stdlib

## Why This Distinction Matters

The question is not whether a module prefix is aesthetically good or bad. The
question is whether a concept has one clear owner. If several layers appear to
own the same idea, source code becomes noisier and tools have to guess which
spelling is intended.

Sigil prefers to solve that problem structurally:

- core for language-shaping concepts
- stdlib for canonical operational modules

That keeps the namespace story smaller and more predictable.
