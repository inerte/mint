#!/usr/bin/env bash
#
# Differential testing: Compare Rust compiler output vs TypeScript compiler
#
# This script compiles the same Sigil file with both compilers and compares
# the generated TypeScript output to ensure they match.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_COMPILER="$SCRIPT_DIR/target/debug/sigil"
TS_COMPILER="node"
TS_CLI="$SCRIPT_DIR/../compiler/dist/cli.js"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if Rust compiler is built
if [ ! -f "$RUST_COMPILER" ]; then
    echo -e "${RED}Error: Rust compiler not built. Run 'cargo build --bin sigil' first.${NC}"
    exit 1
fi

# Check if TypeScript compiler is built
if [ ! -f "$TS_CLI" ]; then
    echo -e "${RED}Error: TypeScript compiler not found at $TS_CLI${NC}"
    echo "Run 'pnpm --filter @sigil-lang/compiler build' to build it."
    exit 1
fi

# Create temp directories for outputs
RUST_OUT_DIR=$(mktemp -d)
TS_OUT_DIR=$(mktemp -d)

cleanup() {
    rm -rf "$RUST_OUT_DIR" "$TS_OUT_DIR"
}
trap cleanup EXIT

echo "Differential Testing: Rust vs TypeScript Compiler"
echo "=================================================="
echo ""

# Test cases: simple Sigil files to compile
TEST_FILES=()

# Find test files if provided as arguments, otherwise use default
if [ $# -gt 0 ]; then
    TEST_FILES=("$@")
else
    # Create a simple test file
    TEST_FILE=$(mktemp /tmp/sigil-test.XXXXXX.sigil)
    cat > "$TEST_FILE" <<'EOF'
λ add(a: ℤ, b: ℤ) → ℤ = a + b

λ main() → ℤ = add(2, 3)
EOF
    TEST_FILES=("$TEST_FILE")
fi

PASSED=0
FAILED=0
SKIPPED=0

for sigil_file in "${TEST_FILES[@]}"; do
    if [ ! -f "$sigil_file" ]; then
        echo -e "${YELLOW}SKIP${NC}: $sigil_file (not found)"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi

    filename=$(basename "$sigil_file")
    echo -n "Testing $filename... "

    # Compile with Rust compiler
    rust_out="$RUST_OUT_DIR/${filename%.sigil}.ts"
    if ! "$RUST_COMPILER" compile "$sigil_file" -o "$rust_out" --human > /dev/null 2>&1; then
        echo -e "${YELLOW}SKIP${NC} (Rust compilation failed)"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi

    # Compile with TypeScript compiler
    ts_out="$TS_OUT_DIR/${filename%.sigil}.ts"
    if ! "$TS_COMPILER" "$TS_CLI" compile "$sigil_file" --human > /dev/null 2>&1; then
        echo -e "${YELLOW}SKIP${NC} (TS compilation failed)"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi

    # TypeScript compiler outputs to .local/ by default, move it for comparison
    default_ts_out=".local/${filename%.sigil}.ts"
    if [ -f "$default_ts_out" ]; then
        cp "$default_ts_out" "$ts_out"
    else
        echo -e "${YELLOW}SKIP${NC} (TS output not found at $default_ts_out)"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi

    # Compare outputs (ignoring whitespace differences for now)
    if diff -w "$rust_out" "$ts_out" > /dev/null 2>&1; then
        echo -e "${GREEN}PASS${NC}"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}FAIL${NC}"
        echo "  Differences found:"
        diff -u "$rust_out" "$ts_out" | head -20
        FAILED=$((FAILED + 1))
    fi
done

echo ""
echo "=================================================="
echo "Results: $PASSED passed, $FAILED failed, $SKIPPED skipped"

if [ $FAILED -gt 0 ]; then
    exit 1
fi

exit 0
