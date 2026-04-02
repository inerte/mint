---
name: sigil-devex-eval
description: Benchmark real Sigil write, edit, and repair outcomes with the developer-experience harness. Use when Codex should compare clean HEAD against the current working tree across the benchmark task suite and report which Sigil tasks got better, worse, or stayed flat.
---

# Sigil Devex Eval

Run the Sigil developer-experience benchmark harness against the current work in
progress or against explicit refs when the user asks.

## Default flow

1. Validate the harness.
2. Run `compare` across the benchmark task suite.
3. Summarize per-task outcomes first, then the overall comparison.
4. Narrow to `--tasks` or explicit refs only when the user asks for a focused run.

Default comparison mode:

- base: clean `HEAD`
- candidate: current working tree snapshot
- tasks: all task manifests

## Commands

Validate:

```bash
pnpm exec tsx language/benchmarks/developer-experience/tools/devex-benchmark.ts validate
```

Compare current work in progress across all tasks:

```bash
pnpm exec tsx language/benchmarks/developer-experience/tools/devex-benchmark.ts compare
```

Focus on selected tasks only:

```bash
pnpm exec tsx language/benchmarks/developer-experience/tools/devex-benchmark.ts compare --tasks <task-id,task-id>
```

Explicit ref compare:

```bash
pnpm exec tsx language/benchmarks/developer-experience/tools/devex-benchmark.ts compare --base <base-ref> --candidate <candidate-ref>
```

Publish a selected run:

```bash
pnpm exec tsx language/benchmarks/developer-experience/tools/devex-benchmark.ts publish --run <run-id> --label <label>
```

## Notes

- Read raw run bundles from `language/benchmarks/developer-experience/.local/runs/`.
- Read tracked summaries from `language/benchmarks/developer-experience/results/`.
- Treat the suite as outcome-first: the benchmark measures whether Codex gets
  better at real Sigil work, not whether a specific internal feature is used.
