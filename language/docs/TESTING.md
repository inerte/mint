# Sigil Testing

Sigil tests are first-class language declarations, not a separate test framework.

## Canonical Layout

- tests live under `tests/`
- `test` declarations outside `tests/` are canonical errors
- test files are ordinary `.sigil` files
- test files may include helpers alongside `test` declarations

Application/library code should live under `src/` and be imported from tests with
normal Sigil imports.

## Importing Real Modules

Library code is file-based, not `export`-based:

```sigil
⟦ src/math.lib.sigil ⟧
λdouble(x:Int)→Int=x*2
```

```sigil
⟦ tests/math.sigil ⟧
i src⋅math

λmain()→Unit=()

test "double 2" {
  src⋅math.double(2)=4
}
```

## Test Syntax

```sigil
test "adds numbers" {
  1+1=2
}
```

Rules:

- test body must evaluate to `Bool`
- `true` passes
- `false` fails

Effectful tests use explicit effects:

```sigil
test "writes log" →!IO {
  console.log("x")=()
}
```

## Mocking

Sigil includes built-in lexical mocking.

Allowed targets:

- extern members
- Sigil functions declared `mockable`

Example:

```sigil
mockable λfetchUser(id:Int)→!Network String="real"

test "fallback on API failure" →!Network {
  withMock(fetchUser, λ(id:Int)→!Network String="ERR") {
    fetchUser(1)="ERR"
  }
}
```

## CLI

Default output mode is JSON.
Human-readable output is available with `--human`.

Examples:

```bash
# Run all tests in the current project tests/ directory
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test

# Run a specific file or subdirectory
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test projects/algorithms/tests/basicTesting.sigil

# Filter by test name substring
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test --match "cache"

# Human-readable output
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test --human
```

For topology-aware projects, `--env <name>` is required.

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
- `durationMs`
- `location`
- `failure` when the test fails or errors

Current aggregated test output does not include:

- `declaredEffects`
- structured `assertion` metadata

Formal references:

- `language/docs/TESTING_JSON_SCHEMA.md`
- `language/spec/cli-json.md`
- `language/spec/cli-json.schema.json`
