#!/bin/bash
# Simple verification: Run examples with Rust compiler and report results

echo "========================================"
echo "Testing Examples with Rust Compiler"
echo "========================================"
echo ""

RUST_COMPILER="./target/debug/sigil"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

passed=0
failed=0
total=0

for file in ../examples/*.sigil; do
    if [ ! -f "$file" ]; then
        continue
    fi

    ((total++))
    name=$(basename "$file")

    # Extract expected output from comment
    expected=$(grep "Expected output:" "$file" | head -1 | sed 's/.*Expected output: //' | awk '{print $1}')

    # Run with Rust compiler
    result=$($RUST_COMPILER run "$file" --human 2>&1)
    exit_code=$?

    # Extract actual output (number before "sigilc run OK")
    actual=$(echo "$result" | grep -oE "^[0-9]+" | head -1)

    if [ $exit_code -eq 0 ] && [ -n "$actual" ]; then
        if [ "$actual" = "$expected" ]; then
            echo -e "${GREEN}✓${NC} $name (output: $actual)"
            ((passed++))
        else
            echo -e "${RED}✗${NC} $name (expected: $expected, got: $actual)"
            ((failed++))
        fi
    else
        echo -e "${YELLOW}!${NC} $name (compilation/runtime error)"
        ((failed++))
    fi
done

echo ""
echo "========================================"
printf "Total: %d | ${GREEN}Passed: %d${NC} | ${RED}Failed: %d${NC}\n" "$total" "$passed" "$failed"
echo "========================================"

if [ $failed -gt 0 ]; then
    exit 1
fi
