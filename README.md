# Sigil Monorepo

This repo contains three distinct things:

- `language/` — the Sigil language implementation (compiler, stdlib, docs, tools)
- `projects/` — canonical Sigil projects and examples
- `website/` — the Sigil website (GitHub Pages target)

## Start Here

- Language/compiler docs: `language/README.md`
- Pure Sigil example project: `projects/algorithms/`
- React + Sigil bridge example: `projects/todo-app/`

## Common Commands

```bash
# Build the compiler
pnpm build

# Run Sigil tests in the algorithms example project
pnpm sigil:test:algorithms

# Run Sigil tests in the todo-app Sigil domain
pnpm sigil:test:todo
```

## Notes

- Sigil user projects should use the canonical layout: `sigil.json`, `src/`, `tests/` (and optional `web/`)
- This monorepo mixes language implementation and projects intentionally, but the user-facing layout is demonstrated under `projects/`
