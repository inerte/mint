---
title: URL Belongs In Stdlib, With One Canonical Surface
date: 2026-03-05
author: Sigil Language Team
slug: stdlib-url-canonical-ownership
---

# URL Belongs In Stdlib, With One Canonical Surface

Sigil now ships `stdlib⋅url`.

This follows the same ownership rule used for `json` and `time`:
- core contains universal language-shaping vocabulary
- stdlib contains operational domains and protocol helpers

`Map` remains core (`{K↦V}` and `core⋅map`).
`url` remains stdlib (`stdlib⋅url`) because it is an operational parsing domain, not foundational language vocabulary.

## Canonical API

`stdlib⋅url` provides one typed surface:
- `parse(input:𝕊)→Result[Url,UrlError]`
- `is_absolute(url:Url)→𝔹`
- `is_anchor(url:Url)→𝔹`
- `get_query(key:𝕊,url:Url)→Option[𝕊]`
- `has_query(key:𝕊,url:Url)→𝔹`
- `suffix(url:Url)→𝕊`

No parallel APIs, no synonyms, no parser-level aliasing.

## Real Usage, Not Just Surface Area

We immediately switched real code paths:
- `projects/ssg` link rewriting now uses `stdlib⋅url.parse` instead of manual string slicing.
- SSG now preserves query + fragment suffixes during internal route rewriting.
- `stdlib⋅http-server` now parses request paths through `stdlib⋅url`.

This keeps Sigil practical while preserving the “one owner, one spelling” model for both humans and LLM agents.
