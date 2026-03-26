---
title: Why Sigil Uses Root Sigils and =>
date: 2026-03-14
author: Sigil Language Team
slug: why-sigil-switched-to-colon-colon-and-fat-arrow
---

# Why Sigil Uses Root Sigils and =>

> Update (2026-03-26): Sigil now also reserves `µ` for project-defined types
> and project sum constructors from `src/types.lib.sigil`. The rest of the
> rooted-reference story below still holds. See
> [/articles/centralized-project-types-and-constrained-type-meanings/](/articles/centralized-project-types-and-constrained-type-meanings/).

Sigil uses explicit root sigils for module provenance and `=>` for function and
match arrows.

## Why Root Sigils

Sigil wants every non-local reference to say where it comes from without
requiring a separate import declaration.

That is why the language uses fixed roots:

- `§` for stdlib modules
- `•` for project source modules
- `¶` for core modules
- `¤` for config modules
- `†` for world modules
- `※` for test modules
- `µ` for project-defined types and project sum constructors

Examples:

- `§list.sum`
- `µOrdering`
- `†runtime.World`
- `※check::log.contains`

This keeps provenance explicit at the use site, avoids local-name collisions,
and removes a whole class of bookkeeping around import declarations.

## Why Not Plain `.`

The strongest token winner for module syntax was collapsing everything into `.`,
but that would have blurred two different relationships:

- `record.field`
- `µOrdering`

Sigil already uses `.` for member access. Keeping the root sigil separate and
using `::` only after the root for deeper module descent preserves a cleaner
grammar boundary.

## Why `=>`

The arrow choice is simpler. `=>` works cleanly for function signatures and
match arms while staying compact and visually regular.

Using one canonical arrow throughout the language keeps the surface easier to
predict for both humans and models.

## Result

This is a good example of Sigil's syntax policy.

The language does not optimize for the shortest isolated glyph sequence. It
chooses the canonical surface that keeps provenance explicit, parsing direct,
and whole-program token cost low.
