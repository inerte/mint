---
title: Why Sigil Enforces Alphabetical Parameter Order
date: 2026-02-27
author: Sigil Team
category: Language Design
tags: [canonical-forms, parameters, determinism]
slug: 008-alphabetical-parameter-ordering
---

# Why Sigil Enforces Alphabetical Parameter Order

Parameter order is usually treated as an API-design choice. Sigil takes a
narrower view. When two functions mean the same thing and differ only in how
their parameters are arranged, the language has created unnecessary room for
variation.

## The Problem

Without a rule, parameter lists drift quickly:

```python
send_email(to, from, subject, body)
send_email(subject, body, to, from)
```

In many languages that is accepted as ordinary style freedom. In Sigil it is a
source of representational choice. The same conceptual operation can be exposed
through several plausible but different orderings, and every new function
definition becomes another place where an author or a generator has to choose.

## The Decision

Sigil enforces alphabetical parameter ordering in declarations. The rule is
simple, mechanical, and easy to check.

The point is not that alphabetical order is somehow semantically superior. The
point is that it is deterministic and external to taste. Once the rule exists,
there is no local argument left to make about where a parameter should go.

## Why This Fits Sigil

This is a stronger constraint than most languages impose, but it lines up with
Sigil's broader design. The language is willing to give up some familiar freedom
when that freedom mostly creates stylistic branching.

Alphabetical ordering is useful here because it removes the placement decision
without introducing a more complicated ranking scheme. The resulting APIs may
look slightly unusual at first, but they are consistent, easy to predict, and
easy for tools to reconstruct.
