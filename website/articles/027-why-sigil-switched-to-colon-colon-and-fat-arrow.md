---
title: Why Sigil Switched to :: and =>
date: 2026-03-14
author: Sigil Language Team
slug: why-sigil-switched-to-colon-colon-and-fat-arrow
---

# Why Sigil Switched to :: and =>

Sigil replaced `⋅` with `::` and `→` with `=>`.

## Why the Change Happened

The decision was based on whole-file tokenization over the actual Sigil corpus,
not on isolated symbol comparisons. Under the tokenizer that matters most for
this repo, the older Unicode forms were more expensive than the ASCII
alternatives.

The measured savings across the `language/` and `projects/` corpus were large
enough to justify a hard break.

## Why `::` Instead of `.`

The strongest token winner for namespace syntax was `.`, but that would have
collapsed an important structural distinction:

- `record.field`
- `fs::promises.readFile`
- `src::graphTypes.Ordering`

Sigil already uses `.` for field access. Reusing it for namespace paths would
make parsing less direct and weaken a grammar boundary the language currently
keeps clean. `::` still improves token cost while preserving that distinction.

## Why `=>`

The arrow change was simpler. `=>` measured slightly better than `->` on the
current corpus and works cleanly across function signatures and match arms.

Using one canonical arrow throughout the language also keeps the surface easier
to predict.

## Result

This change is a good example of Sigil's syntax policy. Token cost matters, but
it is measured on surrounding code and balanced against deterministic parsing.
The language does not simply pick the shortest glyph sequence. It picks the
canonical form that reduces cost without weakening structure.
