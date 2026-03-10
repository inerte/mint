# Sigil Testing (First-Class)

Sigil tests are language declarations, not a separate framework.

## Canonical layout

- Tests must live under `./tests`
- `test` declarations outside `./tests` are compile errors
- Test files may include regular functions/types/constants
- Tests should import real app/library code from `src/...` (typed cross-module imports)

## Testing real modules (`src/...`)

Use canonical Sigil imports and explicit exports in the source module:

```sigil
⟦ src/math.sigil ⟧
export λdouble(x:Int)→Int=x*2
```

```sigil
⟦ tests/math.sigil ⟧
i src⋅math

test "double 2" {
  src⋅math.double(2)=4
}
```

## Test syntax

```sigil
test "adds numbers" {
  1+1=2
}
```

- Test body must evaluate to `Bool`
- `true` passes, `false` fails

## Effectful tests

Use explicit effect annotations on tests (same model as functions):

```sigil
test "writes log" →!IO {
  console.log("x")=()  ⟦ body still must be boolean ⟧
}
```

## Built-in mocking (scoped)

Mocks are explicit, lexical, and automatically restored.

- Allowed targets:
  - `extern` members (e.g. `axios.get`)
  - Sigil functions marked `mockable`
- Not allowed:
  - pure functions
  - non-`mockable` Sigil functions

### `mockable` adapter function

```sigil
mockable λfetchUser(id:Int)→!Network String="real"
```

### `withMock`

```sigil
test "fallback on API failure" →!Network {
  withMock(fetchUser, λ(id:Int)→!Network String="ERR") {
    fetchUser(1)="ERR"
  }
}
```

## CLI

JSON is the default output mode (agent-first).
Test files run in parallel by default (results are sorted deterministically in final output).

```bash
# Run all tests in ./tests
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test

# Run a file or subdirectory under ./tests
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test projects/algorithms/tests/basicTesting.sigil

# Filter by test description substring
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test --match "cache"

# Human-readable output
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- test --human
```

## JSON output (default)

`sigilc test` prints a single JSON object to stdout with:

- `formatVersion`
- `ok`
- `summary`
- `results[]`

Each result includes:

- `id`
- `file`
- `name`
- `status` (`pass`, `fail`, `error`)
- `durationMs`
- `location`
- `declaredEffects`
- `failure` (for failures/errors)

Formal schema:
- `docs/TESTING_JSON_SCHEMA.md` (`formatVersion: 1`)
