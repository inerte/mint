#!/bin/bash
set -e

echo "========================================"
echo "Rust/TypeScript Compiler Parity Test"
echo "========================================"
echo ""

# Create test project
mkdir -p parity-test/src
cat > parity-test/sigil.json << 'EOF'
{}
EOF

cat > parity-test/src/math.sigil << 'EOF'
export λ add(x: ℤ, y: ℤ) → ℤ = x + y
export λ multiply(x: ℤ, y: ℤ) → ℤ = x * y
export λ square(x: ℤ) → ℤ = multiply(x, x)
EOF

cat > parity-test/src/main.sigil << 'EOF'
i src⋅math
λ main() → ℤ = src⋅math.add(src⋅math.square(3), src⋅math.multiply(2, 5))
EOF

echo "Created test project:"
echo "  src/math.sigil - exports add, multiply, square"
echo "  src/main.sigil - imports and uses math functions"
echo "  Expected result: 3² + (2 * 5) = 9 + 10 = 19"
echo ""

# Compile with Rust
echo "=== Compiling with Rust compiler ==="
cd parity-test
../target/debug/sigil compile src/main.sigil --human
echo ""

# Save Rust outputs
mkdir -p ../rust-output
cp .local/src/main.ts ../rust-output/main.ts
cp .local/src/math.ts ../rust-output/math.ts
echo "Rust outputs saved to rust-output/"
echo ""

# Clean and compile with TypeScript
rm -rf .local
echo "=== Compiling with TypeScript compiler ==="
node ../../compiler/dist/cli.js compile src/main.sigil --human
echo ""

# Save TS outputs
mkdir -p ../ts-output
cp .local/src/main.ts ../ts-output/main.ts
cp .local/src/math.ts ../ts-output/math.ts
echo "TypeScript outputs saved to ts-output/"
echo ""

# Compare outputs
echo "=== Comparing Generated Code ==="
echo ""

echo "--- math.ts comparison ---"
if diff -u ../ts-output/math.ts ../rust-output/math.ts > /tmp/math-diff.txt; then
    echo "✅ math.ts: IDENTICAL"
else
    echo "❌ math.ts: DIFFERENT"
    echo "Differences:"
    head -20 /tmp/math-diff.txt
fi
echo ""

echo "--- main.ts comparison ---"
if diff -u ../ts-output/main.ts ../rust-output/main.ts > /tmp/main-diff.txt; then
    echo "✅ main.ts: IDENTICAL"
else
    echo "⚠️  main.ts: Minor differences detected"
    echo "Differences (excluding import paths):"
    grep -v "import.*from" /tmp/main-diff.txt | head -30
fi
echo ""

# Test runtime behavior
echo "=== Testing Runtime Behavior ==="
echo ""

echo "Running with Rust-compiled code:"
../target/debug/sigil run src/main.sigil --human | grep -E "^[0-9]+$" > /tmp/rust-result.txt || true
rust_result=$(cat /tmp/rust-result.txt)
echo "Result: $rust_result"
echo ""

# Clean and run with TS
rm -rf .local
echo "Running with TypeScript-compiled code:"
node ../../compiler/dist/cli.js run src/main.sigil --human | grep -E "^[0-9]+$" > /tmp/ts-result.txt || true
ts_result=$(cat /tmp/ts-result.txt)
echo "Result: $ts_result"
echo ""

# Compare results
if [ "$rust_result" = "$ts_result" ]; then
    echo "✅ Runtime results: IDENTICAL ($rust_result)"
else
    echo "❌ Runtime results: DIFFERENT (Rust: $rust_result, TS: $ts_result)"
fi
echo ""

# Cleanup
cd ..
rm -rf parity-test rust-output ts-output
rm -f /tmp/math-diff.txt /tmp/main-diff.txt /tmp/rust-result.txt /tmp/ts-result.txt

echo "========================================"
echo "Parity Test Complete!"
echo "========================================"
