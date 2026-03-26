#!/bin/bash

# Compile all tracked Sigil source in the repo through the compiler batch path.
# Stops on the first compilation error reported by `sigil compile`.

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color
SCRIPT_DIR="$(cd -- "$(dirname -- "$0")" && pwd)"
SIGIL="$SCRIPT_DIR/language/compiler/target/debug/sigil"
MANIFEST_PATH="$SCRIPT_DIR/language/compiler/Cargo.toml"

echo "═══════════════════════════════════════════════════════════"
echo "  Building Rust compiler"
echo "═══════════════════════════════════════════════════════════"
echo ""

if ! cargo build --quiet --manifest-path "$MANIFEST_PATH" -p sigil-cli 2>&1; then
  echo -e "${RED}Failed to build Rust compiler${NC}"
  exit 1
fi

echo -e "${GREEN}Rust compiler built successfully${NC}"
echo ""

echo "═══════════════════════════════════════════════════════════"
echo "  Compiling all .sigil files in repository"
echo "═══════════════════════════════════════════════════════════"
echo ""

if output=$("$SIGIL" compile . --ignore .git --ignore-from .gitignore); then
  COMPILED=$(echo "$output" | jq -r '.data.summary.compiled')
  MODULES=$(echo "$output" | jq -r '.data.summary.modules')
  DURATION_MS=$(echo "$output" | jq -r '.data.summary.durationMs')
else
  echo -e "${RED}Compilation failed${NC}"
  echo ""
  echo "Error:"
  echo "$output" | jq -r '.error.message' 2>/dev/null || echo "$output"
  echo ""
  echo "═══════════════════════════════════════════════════════════"
  echo -e "${RED}Stopped at first error${NC}"
  echo "═══════════════════════════════════════════════════════════"
  exit 1
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo -e "${GREEN}All files compiled successfully!${NC}"
echo "Total: $COMPILED files"
echo "Modules: $MODULES"
echo "Duration: ${DURATION_MS}ms"
echo "═══════════════════════════════════════════════════════════"
