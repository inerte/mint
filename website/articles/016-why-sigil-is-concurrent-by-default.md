---
title: Async Without Await in Sigil
date: 2026-03-03
author: Sigil Language Team
slug: async-without-await
---

# Async Without Await in Sigil

Sigil keeps one async-capable runtime model without making ordinary expression
structure a concurrency surface.

That means:

- FFI calls and internal Sigil functions compose through one promise-shaped
  model
- users do not write `await` just to keep async code moving
- explicit widening belongs to named concurrent regions such as
  `concurrent urlAudit@5{...}`

That split keeps async plumbing uniform while making batching policy visible in
source.

See:

- `/articles/named-concurrent-regions`
- `/docs/async`

The language docs and specs are the source of truth:

- `language/docs/ASYNC.md`
- `language/docs/syntax-reference.md`
- `language/spec/semantics.md`
