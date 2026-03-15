---
title: One Canonical Naming System
date: 2026-03-07
author: Sigil Team
category: Language Design, Canonical Forms
slug: 021-one-canonical-naming-system
---

# One Canonical Naming System

Sigil now uses one naming system across the language:

- `UpperCamelCase` for types, constructors, and type variables
- `lowerCamelCase` for values, functions, locals, fields, module segments, and
  filenames

## Why This Matters

This is not mainly about aesthetic consistency. It is about reducing ambiguity.

When the shape of a name already tells you whether you are looking at a type or a
value-level symbol, tools and readers get that information cheaply. At the same
time, avoiding a mix of `snake_case`, `kebab-case`, and `camelCase` removes one
more axis of stylistic variance.

## What the Rule Buys

The language gets a simpler naming story:

- type-level names are visually distinct
- value-level names share one canonical style
- imports and filenames align with the same rule

That is a better fit for Sigil than inheriting several naming systems from other
language traditions.
