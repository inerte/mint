---
title: Why Sigil Uses Explicit Generics Without Hindley-Milner
date: 2026-03-03
author: Sigil Language Team
slug: why-sigil-uses-explicit-generics-without-hm
---

# Why Sigil Uses Explicit Generics Without Hindley-Milner

Sigil now supports explicit parametric polymorphism, but it does not adopt
Hindley-Milner as its user model.

## What Sigil Wants from Generics

Generic abstractions such as `Option[T]`, `Result[T,E]`, and generic top-level
helpers are useful because they reduce vocabulary growth. They let many programs
reuse one abstraction instead of multiplying specialized wrappers for each type.

That fits Sigil well.

## What Sigil Does Not Want

What Sigil does not want is implicit let-polymorphism on local bindings. That
style of inference is compact, but it hides behavior behind checker cleverness in
a place where Sigil prefers explicitness.

The language deliberately keeps the rule narrow:

- genericity is declared explicitly
- generic top-level declarations are real
- generic ADTs are real
- local bindings do not silently become polymorphic

## Why This Fits Sigil

This split preserves the useful part of generics without importing a broader
inference model that makes local reasoning less obvious. For Sigil, that is the
important balance: reuse and abstraction are good, but hidden generalization is
not.
