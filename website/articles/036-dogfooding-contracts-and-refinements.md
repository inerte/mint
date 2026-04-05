---
title: Dogfooding Contracts and Refinements in Sigil Projects
date: 2026-04-05
author: Sigil Language Team
slug: dogfooding-contracts-and-refinements
---

# Dogfooding Contracts and Refinements in Sigil Projects

Once `where`, `requires`, and `ensures` became real compile-time proof surfaces,
the next step was obvious: stop talking about them only in isolated snippets and
start using them in Sigil's own examples and projects.

## Runnable Language Examples

The canonical language examples now live in:

- `language/examples/functionContracts.sigil`
- `language/examples/proofMeasures.sigil`

They are deliberately small and compile-clean.

`functionContracts.sigil` shows the call-boundary story:

- a constrained type defined with `where`
- a function body proving an `ensures` clause
- a caller satisfying a `requires` clause through that propagated guarantee

`proofMeasures.sigil` shows the measure and narrowing story:

- `#` participating in refinement checks
- branch-local facts feeding a refined return
- a small contract over result length

Those files are not just documentation. The aggregate repo check now compiles
the full `language/examples/` tree as part of `pnpm sigil:test:all`.

## Project Dogfooding

The proof surfaces now show up in real Sigil project code too.

### Todo App

`projects/todo-app/src/todoDomain.lib.sigil` now uses a contract on
`remainingCount`:

- `requires completed≥0 and total≥completed`
- `ensures result≥0`

This is a good fit for contracts because the important fact lives on a function
boundary rather than in a named domain type.

### Flashcards

`projects/flashcards/src/flashcardsDomain.lib.sigil` now uses contracts on
session helpers:

- `revealAnswer` guarantees `result.revealed`
- `sessionForTopic` guarantees a reset session shape with `currentIndex=0` and
  `¬result.revealed`

Those are exactly the kind of state-transition facts that are awkward to repeat
at every caller and natural to attach to the helper that owns them.

### Algorithms

`projects/algorithms/src/fibonacciSearch.lib.sigil` now carries small proof
facts on helper functions:

- `candidateIndex` guarantees the result stays below `#xs`
- `minIndex` guarantees the returned value stays below both inputs

These are simple, solver-friendly contracts that remove guesswork from helper
calls without turning the algorithm into proof-heavy code.

### Game 2048

`projects/game-2048/src/game2048.lib.sigil` now proves structural invariants
that are easy to state and useful to callers:

- `applyMove` preserves board size
- `emptyGame` guarantees score zero and the requested board size

That is the right level of contract for the project. The solver checks a
meaningful invariant without forcing the code to encode every game rule as a
proof obligation.

### Minesweep

`projects/minesweep/src/minesweepDomain.lib.sigil` now uses contracts where the
facts are local and stable:

- `adjacentBombs` guarantees a non-negative result
- `gameFromBombs` guarantees the resulting board dimensions

Again, the point is not to prove everything. The point is to lock down the
facts the project already depends on.

### Roguelike

The roguelike is where the dogfooding gets broader.

`projects/roguelike/src/types.lib.sigil` now uses constrained alias types for
inventory and counters:

- `InventoryCount`
- `TreasureCount`
- `TurnCount`

Those refined aliases flow through `PlayerLoadout` and `GameState`.

`projects/roguelike/src/roguelike.lib.sigil` then uses small constructor-style
helpers to keep those invariants intact when plain `Int` values are clamped back
into refined counters.

The roguelike also now uses contracts on a few helper functions where the proof
story is direct and stable, such as non-negative result guarantees for ammo and
enemy-count helpers.

## Why `Point` Stayed Plain

One thing did *not* become a constrained type: `Point`.

That was intentional.

Roguelike geometry and pathfinding helpers routinely build intermediate
candidate points that are later filtered by floor checks, bounds checks, actor
occupancy checks, or visibility checks. Making `Point` globally constrained
would encode the wrong invariant and fight the natural shape of the code.

This is the important design lesson:

- use constrained types when the invariant really is global
- use `requires` / `ensures` when the important fact lives on a helper boundary
- leave values plain when temporary out-of-domain intermediates are part of the
  algorithm

Sigil is stricter now, but the goal is still to encode the *right* invariants,
not to force everything through the strongest available surface.
