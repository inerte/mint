---
title: Core vs Stdlib Is About Canonical Ownership, Not Purity
date: 2026-03-04
author: Sigil Language Team
slug: core-vs-stdlib-json-time
---

# Core vs Stdlib Is About Canonical Ownership, Not Purity

Sigil now ships `stdlib::json` and `stdlib::time`.

This is a deliberate ownership decision:
- `Map` stays core (`{K↦V}` and `core::map`) because it is a foundational collection concept.
- `json` and `time` stay stdlib because they are operational domains, not universal language vocabulary.

The important point is not whether a call has a prefix.
The important point is whether there is one canonical owner and one canonical spelling.

## Why This Matters for LLM-First Code

Prefixes are not morally important.
Ambiguity is.

Bad:
- multiple modules exposing overlapping JSON helpers
- half-core/half-stdlib ownership of the same concept
- synonyms that force model guessing

Good:
- one canonical module for JSON (`stdlib::json`)
- one canonical module for time (`stdlib::time`)
- deterministic signatures and typed results (`Result`, `Option`)

## Concrete Outcome

We wired these modules into real projects immediately:
- `projects/ssg` now sorts article dates through strict ISO parsing (`stdlib::time.parseIso`).
- `projects/ssg` now emits `site.json` through `stdlib::json.stringify`.
- `projects/todo-app` now uses `stdlib::decode` to turn raw JSON into trusted exact state values with strict decode errors and tests.

This keeps Sigil practical without blurring core vocabulary boundaries.
