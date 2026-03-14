# Sigil CLI JSON Contract

Sigil CLI commands are machine-first. JSON is the default output mode for:

- `sigilc lex`
- `sigilc parse`
- `sigilc compile`
- `sigilc run`
- `sigilc test`
- `sigilc` usage/unknown-command failures

## Canonical Schema

The normative machine contract is:

- `language/spec/cli-json.schema.json`

Consumers should validate against that schema, not this Markdown file.

## Versioning

- `formatVersion` is the payload format version
- current version: `1`
- backward-incompatible output changes require incrementing `formatVersion`

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

Failures emit:

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

`sigilc test` keeps a specialized top-level `summary` / `results` envelope.

## Diagnostics

Diagnostics are structured and machine-oriented:

- `code`
- `phase`
- `message`
- `location` when available
- `found` / `expected` when useful
- `details`
- `fixits`
- `suggestions`

## Current Notes

The current implementation uses:

- `"sigilc ..."` strings in JSON `command` fields
- no `semanticMap` field in successful `compile` output
- a specialized `test` result shape with `location: {line,column}`

If prose and runtime output disagree, the implementation and
`cli-json.schema.json` are the current source of truth.
