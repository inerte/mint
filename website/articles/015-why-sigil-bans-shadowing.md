---
title: Why Sigil Bans Shadowing
date: 2026-03-03
author: Sigil Language Team
slug: why-sigil-bans-shadowing
---

# Why Sigil Bans Shadowing

Sigil bans local shadowing because a name should not change meaning within the
same lexical region of a program.

## The Problem

Many languages allow an inner binding to reuse an outer name. That can be
convenient, but it also means the same identifier may refer to different values
across nested scopes. The compiler can track that correctly, but it still makes
source harder to read, harder to explain, and easier to alter incorrectly during
automated edits.

For Sigil, that tradeoff is not worth it. Shadowing introduces a representational
choice that mostly benefits local convenience rather than clarity.

## The Decision

Sigil requires fresh names instead of allowing inner scopes to reuse outer ones.
That rule applies across locals, lambda parameters, and pattern bindings.

## Why This Fits Sigil

This is less about scope theory than about source stability. A name should carry
one meaning through the part of the program where it is visible. That is easier
for readers, easier for refactors, and easier for generation tools that work by
editing existing code.

The language gives up a common convenience, but it gains a clearer local naming
model in return.
