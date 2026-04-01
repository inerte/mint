---
title: Machine-First Debugging in Sigil
date: 2026-03-31
author: Sigil Language Team
slug: machine-first-debugging
---

# Machine-First Debugging in Sigil

Sigil's debugger is not built around a human sitting in an IDE and poking at a
live process.

It is built around the thing Sigil already assumes is central: a machine reading
structured state, asking a better next question, and rerunning deterministically
when needed.

## The Shift

Traditional debugging often depends on a few loose tools:

- a stack trace
- ad hoc logs
- rerunning until the bug happens again
- an interactive debugger attached to one live process

That model is workable for a human, but it is lossy for an LLM. The model hides
too much of the state the agent actually needs:

- what phase failed
- what the compiler thought the program meant
- what runtime world was active
- which exact expression failed
- how execution got there
- whether the same run can be reproduced exactly

Sigil now exposes those as first-class surfaces instead.

## The Surfaces

The current debugger story is split deliberately:

- `sigil inspect validate` shows canonical source and validation status
- `sigil inspect types` shows solved top-level types
- `sigil inspect world` shows the normalized runtime world for one env
- `sigil inspect codegen` shows the emitted TypeScript without writing artifacts
- `sigil run --json` returns structured runtime failures
- `sigil run --json --trace` shows execution flow
- `sigil run --json --record` and `--replay` make effectful runs reproducible
- `sigil test` exposes the same debugging surface per test result
- `sigil debug run` and `sigil debug test` provide replay-backed stepping and watches

The point is not “more flags.” The point is that the debugging state is now
owned by the language tooling instead of being reconstructed from informal logs.

## Why Replay Matters

Replay is the part that makes the rest coherent.

Without replay, stepping and repeated inspection are always fragile:

- random values shift
- timer/process/network/file effects move
- the bad run may not happen again

With replay, the debugger can return to the same execution on demand. That
means an LLM can:

- inspect the failing expression
- rerun the same execution with trace
- step through the same run later
- keep watches pinned across snapshots

This is a much better fit for agent workflows than “please reproduce the bug
again while I add more logs.”

## What An LLM Actually Does With It

A useful agent workflow is small and structured:

```bash
sigil run --json --trace src/main.sigil
sigil inspect world . --env test
sigil run --json --record .local/crash.replay.json src/main.sigil
sigil debug run start --replay .local/crash.replay.json --watch state.count src/main.sigil
```

That lets the agent answer four different questions without guessing:

1. Did this fail in parsing, validation, typechecking, topology, or runtime?
2. What exact Sigil expression failed?
3. Was the active world correct?
4. Can I step through the same execution again?

For tests, the shape is similar:

```bash
sigil test --record .local/tests.replay.json tests/
sigil debug test start --replay .local/tests.replay.json --test "tests/cache.sigil::cache hit returns cached value" --watch result.value tests/
```

The important part is that the test id, replay artifact, stepping snapshots, and
watch values are all machine-readable and stable.

## This Is Still A Human Tool

Machine-first debugging does not mean “humans are excluded.”

It means the authoritative debugging data is exposed in a form that machines can
use precisely and humans can still read directly. A human can still inspect the
same JSON, trace, replay artifact, or stepping snapshot. The difference is that
Sigil no longer assumes debugging has to begin with human-only surfaces.

That is the broader Sigil pattern:

- canonical source instead of style drift
- explicit worlds instead of hidden runtime configuration
- replayable effects instead of ad hoc reruns
- exact expression blame instead of vague runtime ownership

## Where The Details Live

This article is only the tour.

For the full debugging workflow, flags, JSON shapes, and examples, use:

- [language/docs/DEBUGGING.md](../language/docs/DEBUGGING.md)
- [language/spec/cli-json.md](../language/spec/cli-json.md)
- [language/docs/TESTING.md](../language/docs/TESTING.md)
