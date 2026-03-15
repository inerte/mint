---
title: One Textual Representation Per Program
date: 2026-03-15
author: Sigil Language Team
slug: one-textual-representation-per-program
---

# One Textual Representation Per Program

Sigil now treats source as a normal form. For any valid AST, there is exactly
one accepted textual representation.

## The Change

Canonicality is now printer-first. The compiler:

1. tokenizes the source
2. parses it into an AST
3. prints the canonical source for that AST internally
4. rejects the file unless the original bytes match that printed form exactly

This is stronger than a style guide and stronger than a formatter-based workflow.
The canonical form is part of the language.

## What This Means

If two source files parse to the same Sigil program, only one of them is valid
Sigil source. `compile`, `run`, and `test` all fail on parseable but
non-canonical text.

There is no public formatter command. The compiler error is the enforcement
point.

## Why Sigil Chose This Model

The language wants canonicality to be part of generation itself, not something
applied after the fact. That matters for tools and coding agents because it
removes stylistic branching at the moment code is produced.

Token density still matters, but it is not the first objective here. The first
objective is that one program has one textual form. Once that invariant is in
place, the printer can still make disciplined choices about where to stay dense
and where to prefer structured multiline output.

## Result

Sigil no longer aims for "one preferred style." It now aims for one accepted
source form per program, enforced directly by the compiler.
