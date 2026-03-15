---
title: Canonical Test Location Enforcement
date: 2026-02-27
author: Sigil Language Team
slug: 010-canonical-test-location-enforcement
---

# Canonical Test Location Enforcement

This article documents an earlier stage of Sigil's test-file rules. Some details
of the surrounding language surface have changed since then, but the core design
pressure is the same: test code should not float freely through the project
layout.

For the current test model, see
[language/docs/TESTING.md](../language/docs/TESTING.md) and
[language/docs/syntax-reference.md](../language/docs/syntax-reference.md).

## The Problem

If test declarations can appear anywhere, file purpose becomes ambiguous.

A Sigil file should tell the compiler, the reader, and the tooling what role it
is playing. Is it an executable module, a library module, or a test file? If
tests can be embedded arbitrarily across the tree, that answer becomes partly a
matter of convention rather than a compiler-visible fact.

## The Decision

Sigil enforced a location rule: test declarations belong under `tests/`
directories, and test files follow the language's test-file expectations rather
than behaving like miscellaneous executable modules.

That gave the repo a clearer structure:

- application or library code lives in the normal source tree
- tests live under `tests/`
- the compiler can validate that separation directly

## Why the Rule Matters

This is not only organizational tidiness. The rule makes file purpose part of
canonical project structure.

That has several effects:

- code generation tools know where test code belongs
- refactors do not have to infer whether an arbitrary file contains tests
- the language's own repository can use one visible testing model

Sigil generally prefers that kind of explicit structure over looser convention.

## Historical Note

The details of Sigil's current testing implementation have continued evolving,
especially now that repo-level integration flows also live as Sigil tests. The
important invariant survived: test code is not just another top-level construct
that can appear anywhere. It is tied to a specific project location and file
role.
