# Sigil Testing

<h2 id="first-class-tests">First-Class Tests</h2>

Sigil tests are first-class language declarations, not a separate test framework.
For the shared debugging workflow across `inspect`, `run`, `test`, replay, and
stepping, see [DEBUGGING.md](./DEBUGGING.md).

Repo-level integration tests are ordinary Sigil test files under
`language/integrationTests/tests/`. They run through the same `sigil test`
machinery as project tests rather than through separate shell launchers.

## Canonical Layout

- tests live under `tests/`
- `test` declarations outside `tests/` are canonical errors
- test files are ordinary `.sigil` files
- test files may include helpers alongside `test` declarations

Application/library code should live under `src/` and be referenced from tests
with rooted module syntax.

## Referencing Real Modules

Library code is file-based, not `export`-based:

```sigil module projects/todo-app/src/todoDomain.lib.sigil
λcompletedCount(todos:[µTodo])=>Int=todos reduce (λ(acc:Int,todo:µTodo)=>Int match todo.done{
  true=>acc+1|
  false=>acc
}) from 0
```

```sigil program projects/todo-app/tests/todoDomain.sigil
λmain()=>Unit=()

test "count completed todos" {
  •todoDomain.completedCount([{done:true,id:1,text:"A"},{done:false,id:2,text:"B"}])=1
}
```

## Test Syntax

```sigil program tests/basic.sigil
λmain()=>Unit=()

test "adds numbers" {
  1+1=2
}
```

Rules:

- test description is a string literal and may span lines
- test body must evaluate to `Bool`
- `true` passes
- `false` fails

In project code, named project types still live in `src/types.lib.sigil` and
tests refer to them through `µ...`.

Effectful tests use explicit effects:

```sigil program tests/writesLog.sigil
λmain()=>Unit=()

test "writes log" =>!Log {
  l _=(§io.println("x"):Unit);
  true
}
```

Tests may also derive a local world:

```sigil program language/tests/testWorld.sigil
λmain()=>Unit=()

test "worlds capture logs" =>!Log world {
  c log=(†log.capture():†log.LogEntry)
} {
  l _=(§io.println("captured"):Unit);
  ※check::log.contains("captured")
}
```

Multiline descriptions use the same string syntax:

```sigil program language/tests/multilineStrings.sigil
λmain()=>Unit=()

test "multiline
test description works" {
  §string.lines("alpha
beta")=["alpha","beta"]
}
```

## Worlds, Observation, and Coverage

Sigil no longer treats tests as code plus ad hoc mocks.

Instead:

- `config/<env>.lib.sigil` exports the baseline `world`
- each `test` may derive that world locally with `world { ... }`
- `†...` builds world entries for `Clock`, `Fs`, `Http`, `Log`, `Process`, `Random`, `Tcp`, and `Timer`
- `※observe::...` exposes raw traces from the active test world
- `※check::...` exposes Bool-returning helpers over those traces

Canonical split:

- `†` is compiler-owned runtime world construction
- `※observe` is raw test-world inspection
- `※check` is ergonomic Bool helpers for tests

Canonical note:

- if a world entry exists only as shared baseline behavior, keep it in
  `config/<env>.lib.sigil.world` instead of restating it in every test

Example:

```sigil program language/tests/testWorld.sigil
λmain()=>Unit=()

test "captured log contains line" =>!Log world {
  c log=(†log.capture():†log.LogEntry)
} {
  l _=(§io.println("captured"):Unit);
  ※check::log.contains("captured")
}
```

`sigil test` also enforces project-surface coverage for project source modules:

- every project `src/*.lib.sigil` function must be executed by the suite
- sum-returning project functions must observe each relevant output variant
- missing surface coverage is reported as ordinary failing test results
- this coverage gate applies to suite-style runs such as `sigil test` or `sigil test path/to/tests/`
- focused single-file runs such as `sigil test path/to/tests/file.sigil` skip the project-wide coverage gate

Library tests may call `•...` exports directly. Executable tests exercise
program behavior through `main`.

## CLI

Default output mode is JSON.
This section keeps the test-specific surface. For the broader debugging
workflow and when to choose `inspect`, `run`, `test`, or `debug`, use
[DEBUGGING.md](./DEBUGGING.md).

Examples:

```bash
# Run all tests in the current project tests/ directory
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test

# Run a specific file or subdirectory
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test projects/algorithms/tests/basicTesting.sigil

# Filter by test name substring
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test --match "cache"

# Trace one test file
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test --trace projects/algorithms/tests/basicTesting.sigil

# Stop the current test when a function is reached
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test --break-fn helper projects/algorithms/tests/basicTesting.sigil

# Record and replay a test run
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test --record .local/tests.replay.json projects/algorithms/tests/basicTesting.sigil
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test --replay .local/tests.replay.json projects/algorithms/tests/basicTesting.sigil

# Start a replay-backed stepping session for one exact test id
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- debug test start --replay .local/tests.replay.json --test "projects/algorithms/tests/basicTesting.sigil::cache hit returns cached value" --watch result.value projects/algorithms/tests/basicTesting.sigil
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- debug test step-into .local/debug/<session>.json

```

For runtime-world projects, `--env <name>` is required.
`sigil test --replay` cannot be combined with `--env`; the replay artifact owns
the resolved per-test world.

For process-heavy harness code, prefer:
- `§process` for child processes
- `§file.makeTempDir` for scratch workspaces
- `§time.sleepMs` for retry loops

## JSON Output

`test` emits a single JSON object to stdout by default.

Top-level fields:

- `formatVersion`
- `command`
- `ok`
- `summary`
- `results`

Each result currently includes:

- `id`
- `file`
- `name`
- `status`
  - `pass | fail | error | stopped`
- `durationMs`
- `location`
- optional `failure`
- optional `trace`
- optional `breakpoints`
- optional `replay`
- optional `exception`

`summary` now also includes:

- `stopped`

Stop-mode breakpoint hits are not runtime errors:

- the current test result becomes `status: "stopped"`
- the suite keeps running later selected tests
- top-level `ok` is still `false`

Replay-backed debug snapshots may also include:

- `watches`
  - ordered results for any configured `--watch local(.field)*` selectors

Current aggregated test output does not include:

- `declaredEffects`
- structured `assertion` metadata
- raw per-test coverage traces

Formal references:

- `language/docs/DEBUGGING.md`
- `language/docs/TESTING_JSON_SCHEMA.md`
- `language/spec/cli-json.md`
- `language/spec/cli-json.schema.json`
