# Sigil Token Benchmarks - Results

**Date:** 2026-03-24  
**Tokenizer:** `tiktoken` (`cl100k_base`)  
**Baseline:** TypeScript  
**Corpus:** 20 published cases: 16 algorithm cases plus 4 language-shaped cases exercising `concurrent`, `world`, and topology-aware code

## Executive Summary

Across the current published corpus, Sigil uses **2014 tokens** where the
TypeScript baseline uses **2541**. That is:

- **20.7% fewer tokens than TypeScript**
- **1.262x TypeScript efficiency** when measured as `TypeScript / Sigil`
- **33.6% fewer characters** than the same TypeScript corpus

Python is still slightly smaller than the TypeScript baseline in this mixed
corpus:

- **2505 tokens total**
- **1.4% fewer tokens than TypeScript**

## Subtotals

| Subcorpus | Cases | Sigil | TypeScript | Python | TS / Sigil | Sigil Fewer Tokens vs TS |
|-----------|------:|------:|-----------:|-------:|-----------:|-------------------------:|
| Algorithms | 16 | 1746 | 2087 | 2113 | 1.195x | 16.3% |
| Language-shaped | 4 | 268 | 454 | 392 | 1.694x | 41.0% |
| **Combined** | **20** | **2014** | **2541** | **2505** | **1.262x** | **20.7%** |

## Per-Case Results

| Case | Category | Sigil | TypeScript | Python | TS / Sigil | Sigil Fewer Tokens vs TS |
|------|----------|------:|-----------:|-------:|-----------:|-------------------------:|
| binary-search | algorithm | 138 | 154 | 154 | 1.116x | 10.4% |
| concurrent-region | language-shaped | 61 | 180 | 152 | 2.951x | 66.1% |
| factorial | algorithm | 44 | 52 | 48 | 1.182x | 15.4% |
| fibonacci | algorithm | 62 | 60 | 60 | 0.968x | -3.3% |
| filter-even | algorithm | 39 | 67 | 70 | 1.718x | 41.8% |
| gcd | algorithm | 14 | 48 | 48 | 3.429x | 70.8% |
| histogram | algorithm | 136 | 151 | 150 | 1.110x | 9.9% |
| insertion-sort | algorithm | 104 | 111 | 109 | 1.067x | 6.3% |
| is-palindrome | algorithm | 24 | 49 | 48 | 2.042x | 51.0% |
| levenshtein-distance | algorithm | 215 | 260 | 256 | 1.209x | 17.3% |
| linear-search | algorithm | 77 | 93 | 93 | 1.208x | 17.2% |
| map-double | algorithm | 44 | 59 | 62 | 1.341x | 25.4% |
| merge-sort | algorithm | 453 | 526 | 533 | 1.161x | 13.9% |
| power | algorithm | 44 | 52 | 52 | 1.182x | 15.4% |
| quick-sort | algorithm | 137 | 145 | 149 | 1.058x | 5.5% |
| sum-list | algorithm | 19 | 50 | 58 | 2.632x | 62.0% |
| topology-http-client | language-shaped | 55 | 70 | 71 | 1.273x | 21.4% |
| topology-http-test-world | language-shaped | 59 | 117 | 87 | 1.983x | 49.6% |
| word-frequency | algorithm | 196 | 210 | 223 | 1.071x | 6.7% |
| world-log-test | language-shaped | 93 | 87 | 82 | 0.935x | -6.9% |
| **Average** | **mixed** | **100.7** | **127.1** | **125.3** | **1.262x** | **20.7%** |
| **Total** | **mixed** | **2014** | **2541** | **2505** | **1.262x** | **20.7%** |

## Current Takeaways

### 1. The root-sigil rewrite materially improved the published corpus

The earlier import-based surface still showed a healthy advantage, but the
current no-import, root-sigil surface moves the mixed published corpus to
**20.7% fewer tokens than TypeScript**.

This is the more relevant number now because it reflects the actual canonical
language surface.

### 2. Sigil now wins 18 of 20 cases

The current TypeScript wins are:

- `fibonacci`: **3.3% fewer tokens** in TypeScript
- `world-log-test`: **6.9% fewer tokens** in TypeScript

Everything else in the active published corpus favors Sigil.

### 3. Language-shaped code is now a stronger advantage than pure algorithms

The most interesting change in the refreshed corpus is the language-shaped
subtotal:

- algorithms: **16.3% fewer**
- language-shaped: **41.0% fewer**

That means the current syntax is not only compact on small algorithmic examples.
It is especially compact on the Sigil-specific surfaces that matter for the
language story: `concurrent`, runtime worlds, and topology-aware configuration.

### 4. The strongest wins are now both small helpers and Sigil-specific surfaces

Largest gains:

- `gcd`: 14 vs 48 tokens
- `concurrent-region`: 61 vs 180 tokens
- `sum-list`: 19 vs 50 tokens
- `is-palindrome`: 24 vs 49 tokens
- `topology-http-test-world`: 59 vs 117 tokens

The new `topology-http-client` result is also notable. Under the old surface it
was a TypeScript win; with the rooted syntax it is now a Sigil win.

### 5. The misses are useful and narrower now

The current misses are:

- `fibonacci`, where helper naming and explicit state threading remain slightly heavier
- `world-log-test`, where a tiny test file still pays a visible observation/check cost

Those are useful constraints. They show where the remaining token overhead lives
without changing the overall conclusion.

## Interpretation

The current benchmark story is:

- Sigil is more token-efficient than TypeScript overall on the current mixed corpus.
- The published gap is now **20.7%**, with explicit algorithm vs language-shaped subtotals.
- The current root-sigil surface is materially better than the older import-based surface.
- Sigil-specific runtime/testing/config code is now one of the strongest parts of the benchmark.
- Python stays close to the TypeScript baseline overall.

## Limitations

- The sample is still small: **20 cases**
- The corpus is mixed, but it is still not a full application benchmark
- This is a token-efficiency benchmark, not a runtime-performance benchmark
- Results should not be overgeneralized to production code without a larger and more varied corpus

## Reproduce

```bash
cd language/benchmarks/tokens/tools
npm install

node language/benchmarks/tokens/tools/compare.js factorial
bash language/benchmarks/tokens/run-all.sh
node language/benchmarks/tokens/tools/unicode-benchmark.js measure --out language/benchmarks/tokens/results/unicode-replacements.json
```
