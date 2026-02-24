# Mint TODO (React + TypeScript Bridge)

This example demonstrates the recommended frontend integration pattern:

- `mint/todo-domain.mint`: canonical Mint domain policy helpers (validation/filter predicates/count rules)
- `src/generated/todo-domain.ts`: generated Mint TypeScript output
- `src/bridge.tsx`: React + localStorage adapter (lintable/prettifiable TypeScript)

## Why a bridge?

Mint stays canonical and deterministic for domain policy.
React stays idiomatic in TypeScript/JSX for UI rendering, list updates, hooks, events, and browser APIs.

## Run

```bash
pnpm install
pnpm mint:compile
pnpm dev
```

## Recompile Mint after changing the domain logic

```bash
pnpm mint:compile
```
