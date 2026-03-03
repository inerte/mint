# Algorithms (Sigil Project)

Canonical pure-Sigil example project.

Layout:
- `sigil.json`
- `src/`
- `tests/`

Implemented algorithms:

- Sorting: `insertion-sort.lib.sigil`, `merge-sort.lib.sigil`
- Number theory: `extended-gcd.lib.sigil`, `sieve-of-eratosthenes.lib.sigil`, `modular-exponentiation.lib.sigil`
- Graphs: `depth-first-search.lib.sigil`, `breadth-first-search.lib.sigil`, `topological-sort.lib.sigil`
- Combinatorics: `permutations.lib.sigil`, `n-queens.lib.sigil`
- Search and distance: `linear-search.lib.sigil`, `levenshtein-distance.lib.sigil`

Supporting modules:

- `graph-types.lib.sigil`
- `graph-helpers.lib.sigil`
- `int-list-helpers.lib.sigil`

Commands (from repo root):

```bash
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- compile projects/algorithms/src/collatz-conjecture.sigil
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test projects/algorithms/tests
```

Phase 1 demo files:

- `src/insertion-sort-demo.sigil`
- `src/merge-sort-demo.sigil`
- `src/extended-gcd-demo.sigil`
- `src/sieve-of-eratosthenes-demo.sigil`
- `src/modular-exponentiation-demo.sigil`
- `src/depth-first-search-demo.sigil`
- `src/breadth-first-search-demo.sigil`
- `src/topological-sort-demo.sigil`
- `src/permutations-demo.sigil`
- `src/n-queens-demo.sigil`
- `src/linear-search-demo.sigil`
- `src/levenshtein-distance-demo.sigil`

Planned next:

- `quicksort`
- `quickselect`
- `k-way-merge`
- `prime-factorization`
- `trial-division-primality`
- `tree-traversals`
- `connected-components`
- `combinations`
- `stable-matching`
- `fibonacci-search`
- `jump-search`
