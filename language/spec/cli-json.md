# Sigil CLI JSON Contract

Sigil CLI commands are machine-first. JSON is the default output mode for:

- `sigilc lex`
- `sigilc parse`
- `sigilc compile`
- `sigilc run`
- `sigilc test`
- `sigilc` (usage/unknown-command failures)

## Canonical Schema

The normative machine contract is:

- `language/spec/cli-json.schema.json`

Consumers should validate against that schema (or compatible generated types), not this Markdown file.

## Versioning

- `formatVersion` is the payload format version.
- Current version: `1`
- Consumers must branch on `formatVersion`.
- Backward-incompatible output changes require incrementing `formatVersion`.

## Common Envelope Pattern

Most commands emit:

```json
{
  "formatVersion": 1,
  "command": "sigilc compile",
  "ok": true,
  "phase": "codegen",
  "data": { "...": "..." }
}
```

Failures use:

```json
{
  "formatVersion": 1,
  "command": "sigilc compile",
  "ok": false,
  "phase": "parser",
  "error": {
    "code": "SIGIL-PARSE-NS-SEP",
    "phase": "parser",
    "message": "invalid namespace separator"
  }
}
```

`sigilc test` currently keeps its historical top-level `summary`/`results` shape (still covered by the shared schema).

## Diagnostics

Diagnostics are structured and machine-oriented:

- `code`
- `phase`
- `message`
- `location` (when available)
- `found` / `expected` (when useful)
- `details` (structured metadata)
- `fixits` (deterministic text edits)
- `suggestions` (machine-readable recovery guidance)

### `fixits` vs `suggestions`

- `fixits`: exact text edits the tool/editor can apply directly
- `suggestions`: non-trivial recovery actions or semantic guidance

Example:
- `SIGIL-PARSE-NS-SEP` may include:
  - a `replace` fixit (`/` -> `⋅`)
  - a `replace_symbol` suggestion explaining the canonical separator

## Validation Guidance

Recommended for tool authors / CI:

1. Parse stdout as a single JSON object.
2. Validate against `language/spec/cli-json.schema.json`.
3. Branch on `formatVersion`.
4. Use `error.code` + `fixits`/`suggestions` for automated recovery loops.

