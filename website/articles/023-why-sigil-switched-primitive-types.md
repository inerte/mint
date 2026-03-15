---
title: Why Sigil Switched Primitive Types
date: 2026-03-10
author: Sigil Team
category: Language Design, Token Efficiency
slug: why-sigil-switched-primitive-types
---

# Why Sigil Switched Primitive Types

Sigil now writes primitive types as `Int`, `Float`, `Bool`, `String`, `Char`,
`Unit`, and `Never` instead of the earlier Unicode glyphs.

## Why the Change Happened

The old spellings were visually distinctive, but they performed poorly on the
tokenizer that matters most in this repo. Whole-file measurements consistently
favored the ASCII forms.

Lowercase and capitalized ASCII performed similarly, so Sigil chose the
capitalized set because it strengthens the language's naming distinction between
type-level and value-level syntax.

## What the Measurements Showed

The benchmark was run on real Sigil files, not isolated symbols. On
`cl100k_base`, the Unicode forms were consistently more expensive than the new
spellings, and the difference was large enough to matter across the corpus.

That made this more than a stylistic cleanup. It was a practical token-efficiency
change with a clear language-design upside.

## Why This Fits Sigil

Sigil is trying to prefer syntax that is cheap, predictable, and visually
coherent. The primitive type switch improved all three at once:

- lower token counts
- better alignment with existing tool priors
- a cleaner distinction between types and values

That combination made the migration straightforward to justify.
