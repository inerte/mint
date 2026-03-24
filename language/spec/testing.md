# Sigil Testing Specification

Version: 1.1.1
Last Updated: 2026-03-24

## Overview

Sigil tests are first-class declarations in the language.

Current implemented testing surface includes:

- top-level `test "name" { ... }`
- optional explicit test effects
- optional `world { ... }` clause on tests
- compiler-owned `world::` and `test::` roots
- CLI test discovery and execution

This spec describes the implemented system in the current compiler, not older
design ideas such as TDAI, semantic maps, coverage mode, or generated tests.

## Test Declaration

Canonical surface:

```sigil program tests/sampleTest.sigil
λmain()=>Unit=()

test "adds numbers" {
  1+1=2
}
```

Effectful test:

```sigil program tests/effectTest.sigil
i stdlib::io

λmain()=>Unit=()

test "writes log" =>!Log  {
  stdlib::io.println("x")=()
}
```

World-derived test:

```sigil program tests/worldTest.sigil
i stdlib::io

i test::check::log

i world::log

λmain()=>Unit=()

test "captured log contains line" =>!Log world {
  c log=(world::log.capture():world::log.LogEntry)
} {
  stdlib::io.println("captured")=() and
  test::check::log.contains("captured")
}
```

Rules:

- test description is a string literal
- test body is an expression block
- test body must evaluate to `Bool`
- `true` means pass
- `false` means fail
- test-local `world { ... }` is declaration-only and contains `c` bindings of `world::...` entry values
- test-local world bindings are visible only inside that test body

## Test Location

`test` declarations are only valid in files under `tests/`.

Test files are ordinary `.sigil` files and may also declare:

- `λ`
- `c`
- `t`
- `i`

Test files are executable-oriented and must define `main`.

## Runtime Worlds

Sigil tests run inside a compiler-owned runtime world.

Baseline world:

- selected from `config/<env>.lib.sigil`
- exported as `c world=(...:world::runtime.World)`

Test-local derivation:

- `test ... world { ... } { ... }` overlays entries onto the selected env world
- singleton entries such as `world::clock.*` or `world::log.*` replace that kind
- topology-indexed `world::http.*` and `world::tcp.*` replace by dependency handle

Observation surface:

- `test::observe::...` exposes raw traces
- `test::check::...` exposes Bool helpers over those traces

Canonical example helpers include:

- `test::observe::http.requests`
- `test::observe::log.entries`
- `test::check::http.calledOnce`
- `test::check::log.contains`

## CLI Surface

Current user-facing command shape:

```bash
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test
```

Common modes:

```bash
# all tests in current project tests/
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test

# one file or directory
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test projects/algorithms/tests

# filter by name substring
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test --match "cache"

```

For runtime-world projects:

- `--env <name>` is required

`sigil test` also enforces project source coverage:

- every function in project `src/*.lib.sigil` must be executed by the suite
- sum-returning project functions must observe each relevant output variant
- missing surface coverage is reported as ordinary failing test results
- suite-style runs (`sigil test`, `sigil test path/to/tests/`) enforce this gate
- focused single-file runs (`sigil test path/to/tests/file.sigil`) skip the project-wide coverage gate

## JSON Output

Default test output is JSON.

Current top-level shape:

- `formatVersion`
- `command`
- `ok`
- `summary`
- `results`
- optional `error`

Per-test results currently include:

- `id`
- `file`
- `name`
- `status`
- `durationMs`
- `location`
- optional `failure`

Current output does not include:

- `declaredEffects`
- assertion metadata
- raw coverage traces

Normative references:

- `language/docs/TESTING_JSON_SCHEMA.md`
- `language/spec/cli-json.md`
- `language/spec/cli-json.schema.json`
