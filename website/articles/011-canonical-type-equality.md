---
title: Canonical Type Equality
date: 2026-03-02
author: Sigil Language Team
slug: canonical-type-equality
---

# Canonical Type Equality

Sigil already cared about canonical source. This change extended the same idea
to type compatibility: aliases and named product types should compare by their
normalized structural form everywhere the checker asks whether two types are the
same.

## The Problem

Before this change, different checker paths could reach slightly different
answers for types that were structurally equivalent. That usually happened when
raw synthesized types were compared directly instead of first normalizing aliases
and named products.

The result was not a dramatic soundness failure. It was something subtler and
more annoying: the same semantic type relationship could behave differently
depending on which checker path you happened to trigger.

## The Decision

Sigil now normalizes aliases and named product types before equality checks
throughout the checker. Sum types remain nominal; this change was specifically
about structural equality for the kinds of types that already conceptually carry
structural meaning.

## Why This Matters

This change reduced a class of checker-path-specific surprises. Once a type is
meant to behave structurally, it should not depend on whether the comparison
happened during annotation checking, function application, field access, or some
other inference path.

That consistency is important for the language in general, and especially
important for tool-driven workflows that depend on stable compiler behavior.

## Result

Type equality now follows the same canonicalization story as the surface
language: equivalent structure should not produce multiple answers depending on
where the compiler looks.
