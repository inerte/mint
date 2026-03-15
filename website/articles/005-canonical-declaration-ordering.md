---
title: Canonical Declaration Ordering
date: 2026-02-24
author: Sigil Language Team
slug: 005-canonical-declaration-ordering
---

# Canonical Declaration Ordering

This article describes an early version of Sigil's declaration-ordering rule.
The current language has changed in a few ways since then, but the design
pressure is still useful to document: Sigil does not want file organization to
be a matter of personal preference.

For the current surface, see
[language/docs/syntax-reference.md](../language/docs/syntax-reference.md) and
[language/docs/CANONICAL_FORMS.md](../language/docs/CANONICAL_FORMS.md).

## The Problem

Most languages leave declaration order mostly unconstrained. Imports may come
first, or types may come first, or functions may be grouped by feature rather
than category. That freedom is common, but it turns file organization into a
small style problem that every codebase must solve on its own.

For Sigil, that is not a harmless preference. If the same file could be arranged
several valid ways, then adding a declaration would require an extra placement
decision every time. Human authors make that decision inconsistently, and code
generation tools do the same.

## The Original Rule

The early compiler enforced a category order for declarations and also required
alphabetical ordering within each category. The point was to make placement
deterministic:

- declarations of the same kind always lived together
- new declarations had an obvious insertion point
- reorganization was not left to taste

That rule reduced stylistic variance and made diffs more predictable.

## Why Ordering Belongs in the Language

There are two common ways to solve declaration ordering:

- leave it to convention or a formatter
- make it a compiler-visible rule

Sigil chose the second path because it fits the language's broader canonicality
story. If order matters only socially, then the language still allows multiple
acceptable source organizations. If order is enforced, the file shape becomes
part of the language contract.

That matters most when code is generated, edited, or repaired automatically. The
tool should not have to infer the local style. The compiler should already know
it.

## Historical Note

The exact category order described in the original article is no longer the
current one. Later changes moved types earlier so that typed FFI declarations
could reference named types cleanly. The later ordering article records that
revision.

The important design point survived the update: Sigil treats declaration order
as part of canonical source, not as a formatting preference.
