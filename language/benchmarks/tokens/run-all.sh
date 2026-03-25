#!/bin/bash
# Run the full published token benchmark corpus from repo root or any cwd.

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cases_manifest="${script_dir}/cases.json"
compare_script="${script_dir}/tools/compare.js"

echo "# Sigil Language - Token Efficiency Benchmarks"
echo ""
echo "Using tiktoken (GPT-4 tokenizer) to count LLM tokens."
echo ""
echo "---"
echo ""

while IFS= read -r case_id; do
  echo ""
  node "$compare_script" "$case_id"
  echo ""
  echo "---"
  echo ""
done < <(node -e 'const fs=require("fs"); const cases=JSON.parse(fs.readFileSync(process.argv[1],"utf8")); Object.keys(cases).sort().forEach((id)=>console.log(id));' "$cases_manifest")

echo ""
echo "# Summary"
echo ""
echo "All benchmarks use tiktoken (GPT-4's tokenizer) for LLM token counting."
echo "Lower token count = more compact = better for LLM training."
