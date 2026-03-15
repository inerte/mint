---
title: Why Sigil Switched to :: and =>
date: 2026-03-14
author: Sigil Language Team
slug: why-sigil-switched-to-colon-colon-and-fat-arrow
---

# Why Sigil Switched to :: and =>

Sigil is a machine-first language.

That means syntax is not only a readability question. It is also a tokenizer
question.

We just replaced:

- `⋅` with `::`
- `→` with `=>`

This was a hard break. No compatibility layer. No dual syntax. One canonical
surface.

## The Measured Win

We measured the change the only way that matters for this repo:

- take the real Sigil corpus
- rewrite syntax in memory
- retokenize whole files with the local tokenizer harness

On the current `language/` and `projects/` Sigil corpus:

- files measured: `214`
- baseline tokenizer: OpenAI `cl100k_base`
- total savings from `⋅` -> `::` and `→` -> `=>`: `4017` tokens

Per change:

- `⋅` -> `::`: `2768` tokens saved
- `→` -> `=>`: `1249` tokens saved

Cross-checks also moved in the same direction:

- SentencePiece/Llama proxy: `3474` tokens saved total
- Anthropic legacy proxy: `3474` tokens saved total

That is enough to matter.

## Why Not Use `.`

The benchmark winner for namespace syntax was not `::`.

It was `.`

But `.` is already field access.

Sigil needs these to stay distinct:

- `record.field`
- `fs::promises.readFile`
- `src::graphTypes.Ordering`

If module paths also used `.`, the parser would have to recover intent from
context in places where the current grammar is clean and deterministic.

That is the wrong trade.

`::` still saves a large number of tokens while preserving a strong structural
boundary between:

- namespace paths
- value field access
- qualified constructors and types

Machine-first does not mean blindly picking the shortest glyph sequence.
It means picking the shortest syntax that still preserves deterministic parsing
and canonical generation.

## Why `=>`

For arrows, the decision was easier.

`=>` measured slightly better than `->` on the corpus, and it keeps a visually
strong separator between:

- parameter list
- effects
- return type
- match arm body

So Sigil now uses one arrow everywhere:

```sigil
λadd(x:Int,y:Int)=>Int=x+y
λidentity[T](x:T)=>T=x
match value{
  Ok(result)=>result|
  Err(error)=>fallback(error)
}
```

That uniformity matters as much as the raw count.

## The Real Principle

The important lesson is not "Unicode bad" or "ASCII good."

The important lesson is:

> token claims must be measured on surrounding code, not on isolated symbols

`⋅` looked elegant.
`→` looked compact.
But whole-file retokenization said there was cheaper syntax available that kept
the language deterministic.

So Sigil changed.

That is what a machine-first language should do.
