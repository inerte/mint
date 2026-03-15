---
title: Topology Is Runtime Truth
date: 2026-03-10
author: Sigil Language Team
slug: topology-is-runtime-truth
---

# Topology Is Runtime Truth

Sigil uses topology files to declare a project's external runtime dependencies.
That is not the same job as configuration, deployment, or orchestration.

## The Split

Topology answers:

- what dependencies exist
- what logical names they have
- which environment names the project recognizes

Configuration answers:

- how a given environment binds those declared dependencies

Keeping those roles separate is the whole point of the topology model.

## Why This Matters

Without that split, one file or one mechanism often ends up mixing several
different concerns:

- dependency identity
- concrete URLs and ports
- environment-specific bindings
- ambient environment-variable access

Sigil treats that as a structural problem. If application code can bypass the
declared dependency model and reach directly for raw endpoints or `process.env`,
then topology stops being the runtime source of truth.

## The Decision

Sigil now pushes runtime structure into explicit project files:

- `src/topology.lib.sigil` declares dependency handles and environments
- `config/<env>.lib.sigil` binds those handles for each environment
- ordinary application code uses topology handles instead of raw endpoints

This makes runtime structure visible to the compiler rather than leaving it
spread across configuration conventions and application code.

## Result

Topology becomes the authoritative declaration of runtime dependencies, config
materializes one environment's bindings, and application code is forced through
the typed handles that connect the two. That is a clearer and more machine-readable
runtime model than ambient configuration alone.
