#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <bundle-root>" >&2
  exit 1
fi

bundle_root="$1"

runtime_dir=""
for candidate in \
  "$bundle_root/runtime/node" \
  "$bundle_root/share/sigil/runtime/node"
do
  if [[ -d "$candidate" ]]; then
    runtime_dir="$candidate"
    break
  fi
done

if [[ -z "$runtime_dir" ]]; then
  echo "failed to locate bundled runtime/node under '$bundle_root'" >&2
  exit 1
fi

required_files=(
  "package.json"
  "pty-runtime.mjs"
  "websocket-runtime.mjs"
  "fswatch-runtime.mjs"
  "sql-runtime.mjs"
)

for required in "${required_files[@]}"; do
  if [[ ! -f "$runtime_dir/$required" ]]; then
    echo "missing required bundled runtime file: $runtime_dir/$required" >&2
    find "$runtime_dir" -maxdepth 1 -type f | sort >&2 || true
    exit 1
  fi
done

if [[ ! -d "$runtime_dir/node_modules" ]]; then
  echo "missing bundled runtime dependencies directory: $runtime_dir/node_modules" >&2
  exit 1
fi

echo "validated bundled runtime helpers under $runtime_dir"
