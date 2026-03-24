# Technical Writer Memory

## Website Articles

Website articles are stored at `website/articles/`

### Article Format

- Use YAML frontmatter with title, date, author, and slug
- Follow the structure of existing articles
- Include clear sections: Problem, Solution, Migration Guide, Benefits
- Use code examples liberally
- End with related documentation links
- Include Claude Code attribution link

### Article Numbering

Latest article: 031-worlds-not-mocks.md
Some numbers have gaps or duplicates (e.g., 007 has two versions, 008 has two versions).
Articles exist from 001 through 031.

### Sigil-Specific Writing Style

- Emphasize the "ONE WAY" philosophy
- Be honest about design mistakes when applicable
- Focus on machine-first, deterministic, canonical forms
- Explain both "what changed" and "why it matters"
- Include migration guides for breaking changes
- Use technical accuracy over marketing language

## Compiler

One compiler, written in Rust at `language/compiler/` (Cargo workspace).

Crates:
- `sigil-ast`
- `sigil-cli`
- `sigil-codegen`
- `sigil-diagnostics`
- `sigil-lexer`
- `sigil-parser`
- `sigil-typechecker`
- `sigil-validator`

No TypeScript compiler exists.

## Projects

Located in `projects/`:
- `algorithms`
- `canonicalStdlibAudit`
- `docsDriftAudit`
- `dungeon-bsp`
- `dungeon-caves`
- `dungeon-random-rooms`
- `homebrewPackaging`
- `ssg`
- `todo-app`
- `topology-http`
- `topology-tcp`

## Key Language Facts

### Naming (compiler-enforced)

- Types: `UpperCamelCase` — `SIGIL-CANON-TYPE-NAME-FORM`
- Constructors: `UpperCamelCase` — `SIGIL-CANON-CONSTRUCTOR-NAME-FORM`
- Type variables: `UpperCamelCase` — `SIGIL-CANON-TYPE-VAR-FORM`
- Functions, params, constants, locals, record fields: `lowerCamelCase` — `SIGIL-CANON-IDENTIFIER-FORM`, `SIGIL-CANON-RECORD-FIELD-FORM`
- Filenames: `lowerCamelCase` — `SIGIL-CANON-FILENAME-CASE`

### Topology / World ABI

Config modules (`config/<env>.lib.sigil`) now export `world` as `c world=(...:world::runtime.World)`.
The older `Bindings` / `stdlib::config.bindings` pattern is no longer the canonical ABI.
World is built through `world::http`, `world::tcp`, and `world::runtime`.
Article 031 covers this: "worlds-not-mocks".

### Tests

- Test declarations only valid under `tests/` (checked: `normalized_path.contains("/tests/")`)
- Error: `SIGIL-CANON-TEST-LOCATION`
- Tests cannot be co-located with source code

### Stdlib Modules

`stdlib::` surface: `config`, `decode`, `file`, `httpClient`, `httpServer`, `io`, `json`, `list`, `numeric`, `path`, `process`, `regex`, `string`, `tcpClient`, `tcpServer`, `time`, `topology`, `url`

`core::` surface: `map`, `option`, `result` (prelude types implicit: `Option`, `Result`, `ConcurrentOutcome`, `Some`, `None`, `Ok`, `Err`, `Aborted`, `Failure`, `Success`)

`stdlib::regex` surface: `compile`, `find`, `isMatch`

### Error Codes

56 total. Categories: Lexer (9), Parser (5), Canonical (29), Typecheck (2), Mutability (1), CLI (8), Runtime (2).
Full list in `language/compiler/ERROR_CODES.md`.

### Primitive Effects

`Clock`, `Fs`, `Http`, `Log`, `Process`, `Tcp`, `Timer`

Named effect aliases only in `src/effects.lib.sigil`, must expand to ≥2 primitives.
