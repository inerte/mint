---
title: Why Sigil Uses and and or
date: 2026-03-02
author: Sigil Language Team
slug: why-sigil-uses-and-and-or
---

# Why Sigil Uses and and or

Sigil now uses the word operators `and` and `or` instead of symbolic logical
operators.

## Why the Symbols Lost

The older symbolic forms were short and visually neat, but they were not strong
enough where Sigil actually evaluates syntax choices:

- token behavior in real programs
- ease of generation by existing coding tools
- consistency with the rest of the surface language

Word operators performed better on those measures and avoided adding more
symbol-heavy syntax where a clear keyword already works well.

## Why This Is the Better Canonical Form

The language is not trying to maximize symbolic density. It is trying to settle
on one practical representation that is cheap to generate and easy to parse
correctly.

`and` and `or` are ordinary words, but in this case that is an advantage rather
than a concession. They reduce friction without weakening the language's
canonicality story.
