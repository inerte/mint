---
title: Unifying Canonical Validation
date: 2026-02-27
author: Sigil Language Team
slug: 007-unifying-canonical-validation
---

# Unifying Canonical Validation

This article describes an earlier stage of Sigil's canonicality pipeline. The
current compiler has continued moving toward a stronger printer-first model, but
the earlier consolidation is still worth documenting because it simplified the
shape of canonical validation before that later shift.

For the current model, see
[language/docs/CANONICAL_ENFORCEMENT.md](../language/docs/CANONICAL_ENFORCEMENT.md)
and
[language/docs/CANONICAL_FORMS.md](../language/docs/CANONICAL_FORMS.md).

## The Problem

Sigil used to split canonical enforcement into two layers:

- a surface validator for whitespace and file-shape rules
- a canonical validator for semantic structure rules

That division was understandable historically, but it made the compiler harder
to reason about. Both stages were enforcing the same broad idea: not every
parseable program should be considered valid Sigil source. The split mostly
forced developers to remember which rule lived in which validator.

## The Decision

The compiler merged those layers into one canonical validation pass. That
consolidation did not change the language goal. It changed where the rules were
owned and how they were reported.

The practical benefits were straightforward:

- one enforcement story instead of two overlapping ones
- clearer diagnostics
- less duplicated traversal logic
- a cleaner base for later canonicality work

## Why This Was Only an Intermediate Step

Unifying the validators was an improvement, but it still left the compiler in a
rule-by-rule validation model. Later work pushed canonicality further by making
the compiler's internal printer the source of truth for accepted surface form.

Even so, this earlier change mattered. It removed an unnecessary structural
split and made later canonicality changes easier to implement.
