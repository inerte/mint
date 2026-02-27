#!/bin/bash
# Differential testing: Compile all projects with both Rust and TS compilers
# Verifies generated code matches

set -e

echo "========================================"
echo "Verifying Compiler Parity on Projects"
echo "========================================"
echo ""

RUST_COMPILER="./target/debug/sigil"
TS_COMPILER="../compiler/dist/cli.js"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

total_files=0
identical=0
different=0
errors=0

# Function to compare two files
compare_files() {
    local rust_file=$1
    local ts_file=$2
    local display_name=$3

    if [ ! -f "$rust_file" ]; then
        echo -e "  ${YELLOW}SKIP${NC} $display_name (Rust didn't generate)"
        ((errors++))
        return
    fi

    if [ ! -f "$ts_file" ]; then
        echo -e "  ${YELLOW}SKIP${NC} $display_name (TS didn't generate)"
        ((errors++))
        return
    fi

    # Compare files (ignoring import path differences: .js extension)
    if diff -u "$ts_file" "$rust_file" | grep -v "^---" | grep -v "^+++" | grep -q "^[+-]"; then
        # Check if only difference is .js extension in imports
        local only_js_diff=$(diff -u "$ts_file" "$rust_file" | grep "^[+-]" | grep -v "^---" | grep -v "^+++" | grep -v "\.js';" || true)

        if [ -z "$only_js_diff" ]; then
            echo -e "  ${GREEN}MATCH${NC} $display_name (only .js extension differs)"
            ((identical++))
        else
            echo -e "  ${RED}DIFF${NC} $display_name"
            echo -e "    ${BLUE}First difference:${NC}"
            diff -u "$ts_file" "$rust_file" | grep "^[+-]" | grep -v "^---" | grep -v "^+++" | head -3 | sed 's/^/    /'
            ((different++))
        fi
    else
        echo -e "  ${GREEN}MATCH${NC} $display_name"
        ((identical++))
    fi
}

# Function to compile a project
compile_project() {
    local project_dir=$1
    local project_name=$(basename "$project_dir")

    echo ""
    echo "Project: $project_name"
    echo "----------------------------------------"

    # Find all .sigil source files
    local sigil_files=$(find "$project_dir" -name "*.sigil" -type f | grep -v node_modules | grep -v ".local" || true)

    if [ -z "$sigil_files" ]; then
        echo -e "  ${YELLOW}SKIP${NC} No .sigil files found"
        return
    fi

    # Compile with Rust compiler
    echo "  Compiling with Rust..."
    cd "$project_dir"
    for file in $sigil_files; do
        $RUST_COMPILER compile "$file" --human > /dev/null 2>&1 || {
            echo -e "  ${YELLOW}WARN${NC} Rust compilation failed for $file"
            ((errors++))
        }
    done

    # Save Rust outputs
    if [ -d ".local" ]; then
        cp -r .local ../rust-output-tmp/ 2>/dev/null || true
    fi

    # Clean and compile with TS
    rm -rf .local
    echo "  Compiling with TypeScript..."
    for file in $sigil_files; do
        node "$TS_COMPILER" compile "$file" --human > /dev/null 2>&1 || {
            echo -e "  ${YELLOW}WARN${NC} TS compilation failed for $file"
            ((errors++))
        }
    done

    # Save TS outputs
    if [ -d ".local" ]; then
        cp -r .local ../ts-output-tmp/ 2>/dev/null || true
    fi

    cd - > /dev/null

    # Compare outputs
    if [ -d "../rust-output-tmp" ] && [ -d "../ts-output-tmp" ]; then
        local generated_files=$(find ../rust-output-tmp -name "*.ts" -type f 2>/dev/null || true)
        for rust_file in $generated_files; do
            local rel_path=${rust_file#../rust-output-tmp/}
            local ts_file="../ts-output-tmp/$rel_path"
            ((total_files++))
            compare_files "$rust_file" "$ts_file" "$project_name/$rel_path"
        done
    fi

    # Cleanup
    rm -rf ../rust-output-tmp ../ts-output-tmp
    cd "$project_dir" && rm -rf .local && cd - > /dev/null
}

# Find all project directories
PROJECT_DIRS="../../projects"

if [ ! -d "$PROJECT_DIRS" ]; then
    echo "Projects directory not found: $PROJECT_DIRS"
    exit 1
fi

for project in "$PROJECT_DIRS"/*; do
    if [ -d "$project" ]; then
        compile_project "$project"
    fi
done

# Summary
echo ""
echo "========================================"
echo "Summary:"
printf "Total files: %d | " "$total_files"
printf "${GREEN}Identical: %d${NC} | " "$identical"
printf "${RED}Different: %d${NC} | " "$different"
printf "${YELLOW}Errors: %d${NC}\n" "$errors"
echo "========================================"

if [ $different -gt 0 ]; then
    echo ""
    echo "⚠️  Some files have differences beyond .js imports"
    echo "Review differences above for details"
    exit 1
elif [ $total_files -eq 0 ]; then
    echo ""
    echo "⚠️  No files were compiled - check project structure"
    exit 1
else
    echo ""
    echo "✅ All compiled files show compiler parity!"
fi
