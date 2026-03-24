---
title: Topology Says What, Config Says How
date: 2026-03-14
author: Sigil Language Team
slug: topology-vs-config
---

# Topology Says What, Config Says How

Sigil draws a strict line between topology and configuration.

- topology says what the program depends on
- config says how one environment binds those dependencies

## Update: Config Now Exports Worlds

As of 2026-03-23, the canonical config surface is `c world=(...:world::runtime.World)`.
That makes config responsible for the whole runtime world of one environment,
not just raw HTTP/TCP bindings. The newer `worlds-not-mocks` article covers the
expanded model and the testing implications.

## Why the Split Matters

When those concerns are mixed together, runtime structure becomes difficult to
reason about. Architectural declarations start carrying local ports, production
URLs, secret names, and other operational details. Application code then begins
to bypass the intended dependency model and reach for raw strings or environment
variables directly.

Sigil treats that as a structural weakness, not a mere style issue.

## The Rule

Topology files declare logical dependencies and environments. Configuration
files build the world for one specific environment around those declared
dependencies. Application code refers to declared handles, not raw endpoints.

The compiler reinforces that split by restricting ambient environment access to
config modules and by requiring explicit `--env` selection for topology-aware
projects.

## Why This Helps

This reduces runtime ambiguity in three ways:

- dependency identity is declared once
- environment-specific binding lives in one predictable place
- application code cannot quietly reconstruct runtime wiring on its own

That makes the project easier to validate and easier to generate against,
because the locations of architectural truth are fixed.
