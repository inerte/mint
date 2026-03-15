---
title: Rewriting the Sigil Compiler in Rust
date: 2026-02-26
author: Sigil Language Team
slug: 009-rust-compiler-rewrite
---

# Rewriting the Sigil Compiler in Rust

Sigil's original compiler was written in TypeScript. That made early iteration
fast, but it also tied distribution and performance to the Node.js runtime. The
Rust rewrite was an attempt to keep the language behavior while changing those
practical constraints.

## Why Rewrite It

Three problems kept coming up with the TypeScript compiler:

- distribution depended on Node.js and a package installation story
- performance was acceptable for early development, but not where we wanted it
  for routine compiler work
- multi-module compilation and deeper compiler evolution were easier to manage in
  a single native binary than in a growing TypeScript toolchain

The rewrite was not motivated by language design aesthetics. It was motivated by
compiler engineering concerns.

## The Goal

The target was not "make a better compiler by changing the language." It was
"preserve the language while improving the implementation platform."

That meant the rewrite needed to keep:

- feature coverage
- observable behavior
- output compatibility where practical

The work only made sense if it reduced runtime and packaging complexity without
turning the language surface into a moving target.

## What Changed

The Rust compiler produced a single binary and removed the language's dependency
on a Node-based compiler runtime. It also improved compile-time performance
enough to justify the rewrite operationally.

Just as importantly, the rewrite clarified the compiler's internal structure.
Phases such as lexing, parsing, validation, type checking, and code generation
became easier to reason about as explicit Rust crates with clear boundaries.

## Why This Matters for Sigil

Sigil is trying to be machine-friendly at the language level, but the compiler
is also part of that story. Installation friction and runtime dependencies
matter, especially for environments where the compiler is invoked frequently by
tools or agents.

The Rust rewrite made the implementation more predictable as a tool, not just as
a codebase. That is the main reason it was worth doing.
