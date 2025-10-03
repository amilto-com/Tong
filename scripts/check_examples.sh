#!/usr/bin/env bash
# Simple regression checker for feature examples
# Usage: from repo root: bash scripts/check_examples.sh
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
EX_DIR="$ROOT_DIR/examples/features"
EXP_DIR="$EX_DIR/expected"
RUST_BIN_DIR="$ROOT_DIR/rust/tong"

FAIL=0

run_example() {
  local file="$1"
  local base=$(basename "$file" .tong)
  local expected="$EXP_DIR/$base.out"
  if [[ ! -f "$expected" ]]; then
    echo "[SKIP] $base (no expected file)" >&2
    return 0
  fi
  echo "[RUN ] $base"
  # Run from Rust binary directory so relative module paths work
  pushd "$RUST_BIN_DIR" > /dev/null
  if ! output=$(cargo run --quiet -- "../../examples/features/$base.tong" 2>&1); then
     echo "$output" | sed 's/^/    /'
     echo "[FAIL] $base (runtime error)" >&2
     FAIL=1
     popd > /dev/null
     return 0
  fi
  popd > /dev/null
  # Normalize line endings
  # Compare
  if diff -u <(printf "%s\n" "$output") "$expected" > /dev/null; then
     echo "[PASS] $base"
  else
     echo "[FAIL] $base (output mismatch)" >&2
     diff -u <(printf "%s\n" "$output") "$expected" || true
     FAIL=1
  fi
}

for f in "$EX_DIR"/*.tong; do
  run_example "$f"
done

if [[ $FAIL -ne 0 ]]; then
  echo "\nSome examples failed." >&2
  exit 1
else
  echo "\nAll examples passed." >&2
fi
