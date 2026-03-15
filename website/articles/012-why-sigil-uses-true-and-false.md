---
title: Why Sigil Uses true and false
date: 2026-03-02
author: Sigil Language Team
slug: why-sigil-uses-true-and-false
---

# Why Sigil Uses true and false

Sigil originally used `⊤` and `⊥` for booleans. The language now uses `true`
and `false` instead.

## Why the Change Happened

The symbolic literals were visually distinctive, but they were a poor fit for
Sigil's actual goals. They cost more under the tokenizers that matter for the
repo, they are less likely to be produced naturally by coding agents, and they
increase the chance of simple parse failures in ordinary editing environments.

None of those costs were offset by a real language-design benefit.

## Why ASCII Won

`true` and `false` have three advantages:

- they are cheaper or comparable under real tokenization
- they align with the priors of existing coding tools and agents
- they remain explicit and easy to read

The important point is not that symbolic booleans are inherently bad. It is that
Sigil wants canonical syntax that works well in the environments where the
language is actually being written and generated.

## Result

Booleans now follow the same general policy as the rest of Sigil's recent syntax
changes: when a mathematically elegant spelling loses on machine-facing
ergonomics and token cost, the language prefers the more practical canonical
form.
