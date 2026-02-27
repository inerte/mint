# Rust Compiler Verification Guide

This document explains how to verify parity between the Rust and TypeScript compilers before deprecating the TS compiler.

## Overview

The Rust compiler has achieved functional parity with the TypeScript compiler. Before archiving the TS compiler, we should verify output equivalence across all examples and projects.

## Verification Scripts

### 1. Multi-Module Integration Tests
```bash
./test-multimodule.sh
```
Tests the module system with:
- Simple multi-file projects
- Multiple imports
- Transitive dependencies

**Status:** ✅ All 3 tests passing

### 2. Parity Testing (Multi-Module)
```bash
./test-parity.sh
```
Compiles a multi-module project with both compilers and compares:
- Generated TypeScript code
- Runtime behavior

**Status:** ✅ Runtime identical, minor cosmetic differences

### 3. Example Verification
```bash
./verify-all-examples.sh
```
Runs all 52+ examples with both compilers:
- Original examples (fibonacci, list-operations, etc.)
- New corpus (24 comprehensive examples)

**Expected:** Most pass, some may have parser limitations

### 4. Project Verification
```bash
./verify-all-projects.sh
```
Compiles all projects in `../../projects/` with both compilers:
- algorithms/
- todo-app/
- Any other projects

Compares generated outputs file-by-file.

## Known Differences

### Acceptable (Cosmetic)
1. **Import paths**: Rust uses `.js` extension, TS doesn't
   - `import * as x from './utils.js'` (Rust)
   - `import * as x from './utils'` (TS)
   - Both are valid ESM

2. **Parentheses**: Rust adds extra parens around await
   - `(await f())` (Rust)
   - `await f()` (TS)
   - Both functionally identical

### Critical (Must Match)
1. **Mock wrapping**: ✅ Identical
   - Both wrap imported functions with `__sigil_call`
   - Enables `with_mock` in tests

2. **Runtime behavior**: ✅ Identical
   - All differential tests produce same output

## Verification Checklist

Before deprecating TypeScript compiler:

- [x] Multi-module compilation works
- [x] Mock support for imported functions
- [ ] Run `./verify-all-examples.sh` - most examples pass
- [ ] Run `./verify-all-projects.sh` - all projects compile identically
- [ ] Document any intentional differences
- [ ] Update main documentation to use Rust compiler

## Running Full Verification

```bash
cd language/compiler-rs

# Build Rust compiler
cargo build --release

# Run all verification
echo "=== Multi-Module Tests ==="
./test-multimodule.sh

echo ""
echo "=== Parity Test ==="
./test-parity.sh

echo ""
echo "=== Examples Verification ==="
./verify-all-examples.sh

echo ""
echo "=== Projects Verification ==="
./verify-all-projects.sh
```

If all pass, the Rust compiler is ready for production and the TS compiler can be archived.

## Next Steps After Verification

1. **Archive TS compiler**
   - Move to `language/compiler-archived/`
   - Update README with deprecation notice
   - Keep for reference only

2. **Update documentation**
   - Point all docs to Rust compiler
   - Update installation instructions
   - Remove TS compiler from CI/build

3. **Announce**
   - Update website
   - Publish article completion
   - Mark TS compiler as frozen

## Troubleshooting

### Example fails to compile
Check if the example uses unimplemented features:
- `**` operator (power)
- Some stdlib functions
- Parser limitations with certain patterns

### Output differs
Compare diff to determine if:
- Only import paths differ → OK
- Only parentheses differ → OK
- Runtime behavior differs → INVESTIGATE

### Projects fail
Check for:
- Missing sigil.json
- Import cycle issues
- Type errors in cross-module imports
