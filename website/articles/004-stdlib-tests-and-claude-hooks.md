---
title: Stdlib Tests and Claude Hooks
date: 2026-02-25
author: Sigil Language Team
slug: 004-stdlib-tests-and-claude-hooks
---

# Stdlib Tests and Claude Hooks

This change combined two related cleanup steps: turning ad hoc stdlib demo files
into real Sigil tests, and wiring repo automation so those tests run after
relevant edits. The goal was not just better test hygiene. It was to make the
repository follow the same language rules it asks users and tools to follow.

## The Problem

The repo had files under `language/stdlib/` with names that looked like tests,
but they were actually small executable demos built around `λmain()` and console
output. That created a mismatch between Sigil's stated testing model and the way
the stdlib area was exercised in practice.

For a language that pushes canonical structure, that inconsistency matters. The
repo is part of the language surface that people and agents learn from.

## The Decision

The solution was to create a dedicated `language/stdlib-tests/` project and move
behavioral checks there as proper `test` declarations under `tests/`.

That gave the stdlib a cleaner split:

- `language/stdlib/` contains library modules
- `language/stdlib-tests/` contains consumer-facing behavior tests

This matches Sigil's broader rule that tests live under `tests/` and use the
language's native test syntax rather than ad hoc executable programs.

## Why the Hook Matters

Once those tests existed, the next problem was feedback timing. Manual test runs
work, but they are easy to skip during rapid compiler or stdlib edits,
especially in an AI-assisted workflow.

The Claude hook closes that gap:

1. a relevant file is edited
2. the hook inspects the path
3. stdlib tests run automatically when the change is likely to matter

Path filtering is important here. Running the stdlib suite after every edit
would make the automation noisy and easy to ignore. Restricting it to the stdlib
and compiler paths keeps the signal higher.

## Why This Fits Sigil

This change did not introduce a new language feature, but it tightened the
relationship between the language and the repo that hosts it. Sigil now uses its
own testing model for the stdlib, and the automation reinforces that model
during day-to-day development.

That is the right kind of dog-fooding. It does not just prove that the feature
exists. It forces the repository to depend on it continuously.
