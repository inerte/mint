---
title: "Typed FFI and the t→e→i Declaration Order Change"
date: February 25, 2026
author: Sigil Language Team
slug: typed-ffi-and-declaration-ordering
---

# Typed FFI and the t→e→i Declaration Order Change

> **🚨 BREAKING CHANGE:** Sigil's canonical declaration ordering changed from `e → i → t → c → λ → test` to **`t → e → i → c → λ → test`** (types now come first). This enables typed FFI declarations to reference named types. Migration is straightforward: move type declarations before extern/import declarations.

**TL;DR:** We added typed FFI declarations (compile-time type checking at FFI boundaries) and changed canonical declaration ordering to support them. Types must now come before externs because typed extern declarations can reference named types. This is a breaking change but affects very few files and the compiler gives clear fix-it instructions.

## The Problem: Untyped FFI Was Too Permissive

Before typed FFI, all external function calls used TypeScript's `any` type:

```sigil
⟦ Before: Untyped FFI ⟧
e fs⋅promises

λensureDir(dir:𝕊)→𝕌={
  ⟦ What type is opts? The compiler doesn't know! ⟧
  l opts = {recursive:⊤};
  fs⋅promises.mkdir(dir, opts)  ⟦ No type checking here ⟧
}
```

**Problems:**
1. No compile-time type checking on FFI calls
2. Operators like `#` (length) don't work on FFI results because type is `any`
3. Can't reference Sigil types in FFI signatures
4. IDE/LSP can't provide type information

The original issue: `fs⋅promises.readdir` returned `any`, so you couldn't use `#files` to get the length—even though readdir returns a list!

## The Solution: Typed FFI Declarations

We added **optional type annotations** for extern declarations:

```sigil
⟦ After: Typed FFI ⟧
t MkdirOptions = { recursive: 𝔹 }

e fs⋅promises : {
  mkdir : λ(𝕊, MkdirOptions) → 𝕌
}

c opts:MkdirOptions={recursive:⊤}

λensureDir(dir:𝕊)→𝕌=
  fs⋅promises.mkdir(dir, opts)  ⟦ Now type-checked! ⟧
```

**Benefits:**
- ✅ Compile-time type checking at FFI boundaries
- ✅ Can reference **named Sigil types** (like `MkdirOptions`)
- ✅ Operators work on typed results (`#` on lists, `?` on options)
- ✅ Better IDE/LSP support
- ✅ Self-documenting external APIs
- ✅ Backward compatible: untyped `e console` still works

### Typed FFI Syntax

```sigil
⟦ Untyped FFI (trust mode) ⟧
e console

⟦ Typed FFI (type-safe mode) ⟧
e console : {
  log : λ(𝕊) → 𝕌,
  error : λ(𝕊) → 𝕌
}
```

Each member gets a function type signature. The compiler type-checks all calls against these signatures.

### Example: List Types Enable Operators

```sigil
⟦ Typed FFI with list return type ⟧
e fs⋅promises : {
  readdir : λ(𝕊) → [𝕊]
}

λcountFiles(dir:𝕊)→ℤ={
  l files = fs⋅promises.readdir(dir);
  #files  ⟦ Now works! Type is [𝕊], not any ⟧
}
```

Before typed FFI: `#files` would fail because type was `any`.

After typed FFI: `#files` works because type is `[𝕊]`.

## The Ordering Problem

Here's where it gets interesting. **Typed FFI needs to reference named types:**

```sigil
t MkdirOptions = { recursive: 𝔹 }

e fs⋅promises : {
  mkdir : λ(𝕊, MkdirOptions) → 𝕌  ⟦ References the type above ⟧
}
```

But Sigil's original canonical ordering was `e → i → t` (externs before types).

**This created a conflict:**
- Externs can reference types (for typed FFI)
- But types came after externs in canonical order
- So you couldn't write canonical typed FFI!

### The Wrong Solution: Multi-Pass Type Resolution

We could have made the typechecker smarter:

```typescript
❌ Wrong approach: Complex multi-pass resolution
1. First pass: Collect all type declarations
2. Second pass: Resolve extern type references
3. Third pass: Type check function bodies
```

This would work, but it violates Sigil's core principle: **canonical code over implementation complexity**.

**Why this is wrong:**
- Adds implementation complexity
- Makes declaration order semantically meaningless
- Doesn't match how humans read code (top-to-bottom)
- Breaks Sigil's "one way to do it" philosophy

## The Canonical Solution: t→e→i Ordering

Instead, we **changed the canonical declaration order** to match dependency flow:

```
BEFORE (Feb 24, 2026):  e → i → t → c → λ → test
AFTER  (Feb 25, 2026):  t → e → i → c → λ → test
```

**Rationale:**
1. **Types come first** - They're the most fundamental declarations
2. **Externs can reference types** - For typed FFI signatures
3. **Imports can reference types** - For module type dependencies
4. **Functions can reference everything** - They come after all declarations they might use

This follows Sigil's core design principle: **fix with canonical syntax, not implementation workarounds**.

### Top-to-Bottom Dependency Flow

```sigil
⟦ 1. Types - Most fundamental ⟧
t MkdirOptions = { recursive: 𝔹 }
t User = { name: 𝕊, age: ℤ }

⟦ 2. Externs - Can reference types ⟧
e fs⋅promises : {
  mkdir : λ(𝕊, MkdirOptions) → 𝕌
}

⟦ 3. Imports - Can reference types ⟧
i stdlib⋅list

⟦ 4. Consts - Can reference types ⟧
c DEFAULT_USER:User={name:"Guest",age:0}

⟦ 5. Functions - Can reference everything ⟧
λensureDir(dir:𝕊)→𝕌=
  fs⋅promises.mkdir(dir, {recursive:⊤})
```

Everything flows **top-to-bottom**. Types are declared first because everything else can reference them.

## Migration Guide

The compiler gives **clear, actionable error messages** when it finds wrong ordering:

### Error Message

```
Canonical Ordering Error: Wrong category position

Found: e (extern) at line 5
Expected: extern declarations must come before type declarations

Category order: t → e → i → c → λ → test
  t    = types
  e    = externs (FFI imports)
  i    = imports (Sigil modules)
  c    = consts
  λ    = functions
  test = tests

Move all extern declarations to appear before type declarations.

Sigil enforces ONE way: canonical declaration ordering.
```

### Step-by-Step Migration

**1. Identify files with types and externs**

```bash
# Find files that need migration
grep -l "^e " *.sigil | xargs grep -l "^t "
```

**2. Move type declarations before externs**

Before (wrong order):
```sigil
e console
e fs⋅promises

t MkdirOptions = { recursive: 𝔹 }

λmain()→𝕌=console.log("hi")
```

After (correct order):
```sigil
t MkdirOptions = { recursive: 𝔹 }

e console
e fs⋅promises

λmain()→𝕌=console.log("hi")
```

**3. Compile to verify**

```bash
node language/compiler/dist/cli.js compile your-file.sigil
```

If you still get ordering errors, read the error message—it tells you exactly what to fix.

### Automated Migration (Optional)

The compiler error messages are so specific that you can usually fix files in seconds. For large codebases:

```bash
# Future tool (not yet implemented):
sigil fmt --fix-ordering *.sigil
```

## Real-World Impact

We updated the entire Sigil codebase (60+ files). Here's what we found:

**Files affected:**
- ✅ 4 files needed type/extern reordering
- ✅ 3 stdlib FFI modules (ffi_node_*.sigil)
- ✅ All examples with typed FFI

**Migration time:** ~5 minutes total

**Files unchanged:** 95%+ of files don't use both types and externs, so no changes needed.

### Example: test-typed-ffi-mkdir.sigil

Before (would fail compilation):
```sigil
⟦ ERROR: extern references type that comes after it ⟧
e fs⋅promises : {
  mkdir : λ(𝕊, MkdirOptions) → 𝕌  ⟦ MkdirOptions not defined yet! ⟧
}

t MkdirOptions = { recursive: 𝔹 }

c opts:MkdirOptions={recursive:⊤}

λensureDir(dir:𝕊)→𝕌=
  fs⋅promises.mkdir(dir, opts)
```

After (canonical and correct):
```sigil
⟦ Define the type first ⟧
t MkdirOptions = { recursive: 𝔹 }

⟦ Extern can reference it ⟧
e fs⋅promises : {
  mkdir : λ(𝕊, MkdirOptions) → 𝕌
}

⟦ Use it ⟧
c opts:MkdirOptions={recursive:⊤}

λensureDir(dir:𝕊)→𝕌=
  fs⋅promises.mkdir(dir, opts)
```

**The fix:** Move `t MkdirOptions` from line 7 to line 1. That's it.

## Why This Is The Right Choice

### Canonical Syntax > Implementation Complexity

Sigil could have supported `e → i → t` ordering with multi-pass type resolution. Many languages do this.

**But Sigil is different:**
- Designed for **AI code generation** (deterministic, canonical forms)
- Optimized for **machine-first workflows** (clear rules, no magic)
- Built on **"ONE way to do it"** (zero flexibility, maximum clarity)

When faced with a choice between:
1. **Add implementation complexity** (multi-pass type resolution)
2. **Fix the canonical ordering** (types before externs)

Sigil chooses #2 every time.

### Declaration Order Reflects Dependency Flow

The new ordering matches how code is read:

```
Types:     Define the data structures
  ↓
Externs:   Import external functions (using types)
  ↓
Imports:   Import Sigil modules (using types)
  ↓
Consts:    Define constants (using types)
  ↓
Functions: Implement logic (using everything)
  ↓
Tests:     Verify behavior (using everything)
```

**Top-to-bottom dependency flow.** No forward references needed for types.

### Comparison: Other Languages

**TypeScript/JavaScript:** Declarations can be in any order. Result: style wars, linter rules, bike-shedding.

**Python:** Declarations can be in any order (runtime evaluated). Result: import ordering bugs.

**Go:** `gofmt` enforces import ordering. **Better!** But types/functions can be in any order.

**Rust:** `rustfmt` enforces some ordering. **Better!** But incomplete.

**Sigil:** Compiler enforces **complete canonical ordering**. Types before externs before imports before consts before functions before tests. Always. Zero exceptions.

## Typed FFI Examples

### Basic Type Checking

```sigil
⟦ Define the signature ⟧
e console : {
  log : λ(𝕊) → 𝕌
}

⟦ This works - correct type ⟧
λgood()→𝕌=console.log("type safe")

⟦ This fails - type error ⟧
λbad()→𝕌=console.log(42)  ⟦ ERROR: Expected 𝕊, got ℤ ⟧
```

The compiler catches type errors **before runtime**!

### Complex Types with Named Records

```sigil
⟦ Define option types ⟧
t ReadFileOptions = { encoding: 𝕊, flag: 𝕊 }
t WriteFileOptions = { encoding: 𝕊, mode: ℤ }

⟦ Typed extern using those types ⟧
e fs⋅promises : {
  readFile : λ(𝕊, ReadFileOptions) → 𝕊,
  writeFile : λ(𝕊, 𝕊, WriteFileOptions) → 𝕌
}

⟦ Type-checked FFI calls ⟧
λreadConfig(path:𝕊)→𝕊={
  l opts=ReadFileOptions{encoding:"utf8",flag:"r"};
  fs⋅promises.readFile(path, opts)
}
```

**Benefits:**
- Type safety at FFI boundaries
- Clear documentation of external API expectations
- Compile-time error checking
- No runtime surprises

### List Types Enable Operators

```sigil
e fs⋅promises : {
  readdir : λ(𝕊) → [𝕊]
}

λcountFiles(dir:𝕊)→ℤ={
  l files=fs⋅promises.readdir(dir);
  #files  ⟦ Works! Type is [𝕊] ⟧
}

λlogFiles(dir:𝕊)→𝕌={
  l files=fs⋅promises.readdir(dir);
  l count=#files;
  console.log("Found " + count + " files")
}
```

**Before typed FFI:** `#files` would fail (type `any`).

**After typed FFI:** `#files` works (type `[𝕊]`).

## Standard Library FFI Modules

We created typed FFI modules for common Node.js APIs:

### stdlib/ffi_node_console.sigil

```sigil
⟦ Typed console operations ⟧
e console : {
  log : λ(𝕊) → 𝕌,
  error : λ(𝕊) → 𝕌,
  warn : λ(𝕊) → 𝕌
}

export λlog(msg:𝕊)→𝕌=console.log(msg)
export λerror(msg:𝕊)→𝕌=console.error(msg)
export λwarn(msg:𝕊)→𝕌=console.warn(msg)
```

### stdlib/ffi_node_fs.sigil

```sigil
⟦ Typed file system operations ⟧
e fs⋅promises : {
  readFile : λ(𝕊, 𝕊) → 𝕊,
  writeFile : λ(𝕊, 𝕊) → 𝕌,
  readdir : λ(𝕊) → [𝕊],
  mkdir : λ(𝕊, {recursive:𝔹}) → 𝕌
}

export λreadFile(path:𝕊)→𝕊=
  fs⋅promises.readFile(path, "utf8")

export λwriteFile(path:𝕊, content:𝕊)→𝕌=
  fs⋅promises.writeFile(path, content)
```

These modules provide **type-safe wrappers** around Node.js APIs.

## Implementation Details

### Parser Extension

Extended `ExternDecl` AST node with optional `members` field:

```typescript
interface ExternDecl {
  type: 'ExternDecl';
  modulePath: string;
  members?: { [name: string]: FunctionTypeExpr };  // NEW
  location: Location;
}
```

### Typechecker Changes

When checking FFI member access:

```typescript
if (externDecl.members) {
  // Typed FFI - type check against declared signature
  const memberType = externDecl.members[memberName];
  checkCallTypeMatches(callSite, memberType);
} else {
  // Untyped FFI - trust mode (any)
  return anyType;
}
```

### Grammar Update

```ebnf
ExternDecl = "e" ModulePath (":" "{" ExternMembers "}")?

ExternMembers = ExternMember ("," ExternMember)* ","?

ExternMember = Identifier ":" FunctionTypeExpr

FunctionTypeExpr = "λ" "(" TypeList? ")" "→" Type
```

## Breaking Change Summary

### What Changed

1. Canonical declaration order: `e → i → t` became `t → e → i`
2. Compiler enforces types before externs before imports
3. Error messages guide migration

### Who Is Affected

**Affected:** Files that have BOTH type declarations AND extern/import declarations.

**Not affected:**
- Files with only functions (95%+ of user code)
- Files with only externs (no types)
- Files with only types (no externs)
- Generated code (compiler already outputs canonical order)

### Migration Difficulty

**Difficulty:** LOW

**Time:** 1-5 minutes per file

**Process:**
1. Run compiler
2. Read error message
3. Move type declarations above externs
4. Recompile

**The compiler tells you exactly what to fix.**

## Future Work

### Auto-Formatter

```bash
# Planned feature:
sigil fmt --fix-ordering *.sigil
```

Automatically reorder declarations to canonical form.

### LSP Integration

Real-time highlighting of ordering violations in your editor.

### More Typed FFI Modules

Expand stdlib with typed FFI wrappers for:
- `node:path`
- `node:process`
- `node:http`
- Popular NPM packages

### Type Inference for FFI

```sigil
⟦ Future: Infer FFI types from TypeScript .d.ts files ⟧
e axios  ⟦ Automatically typed from @types/axios ⟧
```

## Conclusion

Typed FFI brings **compile-time type safety** to Sigil's foreign function interface. No more `any` types at FFI boundaries. No more runtime type surprises.

The `t → e → i` declaration ordering change enables this feature while maintaining Sigil's core principles:

1. **Canonical code** - ONE way to organize declarations
2. **Machine-first** - Clear rules, no magic
3. **Deterministic** - Same input → same output

We chose to **fix the syntax** (reorder declarations) rather than **add implementation complexity** (multi-pass type resolution). This is the Sigil way.

**Breaking change?** Yes.

**Worth it?** Absolutely.

**Migration difficulty?** Minimal (compiler guides you).

**Result:** Type-safe FFI that references named Sigil types, with canonical top-to-bottom dependency flow.

---

## Related Documentation

- [FFI Documentation](/language/docs/FFI.md) - Complete FFI reference
- [Canonical Declaration Ordering](/articles/005-canonical-declaration-ordering) - Original ordering article (updated)
- [Machine-First Language Design](/about/philosophy) - Sigil's design principles

## Try It Yourself

```bash
# Clone the repo
git clone https://github.com/sigil-lang/sigil.git

# Build the compiler
pnpm install
pnpm --filter @sigil-lang/compiler build

# Try typed FFI examples
node language/compiler/dist/cli.js run language/examples/typed-ffi-demo.sigil
node language/compiler/dist/cli.js run language/test-fixtures/test-typed-ffi-mkdir.sigil
```

**See the error messages:**
```bash
# This will show the ordering error
node language/compiler/dist/cli.js compile test-wrong-order.sigil
```

**Read the code:**
- Typed FFI parser: `language/compiler/src/parser/parser.ts`
- Canonical validator: `language/compiler/src/validator/canonical.ts`
- Typechecker integration: `language/compiler/src/typechecker/bidirectional.ts`

---

**ONE canonical order. Type-safe FFI. Deterministic code generation.**

This is Sigil.
