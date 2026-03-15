---
title: Canonical Filename Validation
date: 2026-02-27
author: Sigil Team
category: Language Design, Canonical Forms
slug: 006-canonical-filename-validation
---

# Canonical Filename Validation

Sigil now treats filenames as part of canonical source structure rather than as
an external project convention. Module filenames must use `lowerCamelCase`
stems.

Examples of valid names include:

- `userService.lib.sigil`
- `example01Introduction.sigil`
- `ffiNodeConsole.lib.sigil`

Invalid forms include:

- `UserService.sigil`
- `user_service.sigil`
- `user-service.sigil`
- `01-introduction.sigil`

## Why This Rule Exists

If the language wants one canonical naming system, filenames cannot be left to a
mix of `snake_case`, `kebab-case`, and `camelCase`. Those differences create
avoidable variance in imports, project layout, and generated code.

The filename rule keeps module names aligned with the rest of Sigil's value-level
naming story:

- predictable import paths
- less style drift across repos
- fewer filesystem-specific surprises
- less ambiguity for tools and agents

This is a small rule, but it fits the same general pattern as the rest of
Sigil's canonicality work: remove representational choice where it does not buy
anything meaningful.
