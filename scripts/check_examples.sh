#!/usr/bin/env bash
# Comprehensive regression checker for all non-SDL examples.
# Usage: bash scripts/check_examples.sh
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

FAIL=0
TOTAL=0
PASS=0

FILES=()
while IFS= read -r -d '' f; do
  FILES+=("$f")
done < <(find examples -type f -name '*.tong' -print0 | sort -z)

for f in "${FILES[@]}"; do
  if [[ $f == examples/modules/sdl/* ]]; then
    continue
  fi
  rel="${f#examples/}"
  rel_no_ext="${rel%.tong}"
  expected="examples/expected/${rel_no_ext}.out"
  # Use pre-increment to avoid set -e abort (post-increment returns previous value, which can be 0 => exit 1)
  (( ++TOTAL ))
  echo "[RUN ] $rel"
  tmp_out="$(mktemp)"
  if ! cargo run --quiet --manifest-path rust/tong/Cargo.toml -- "$f" > "$tmp_out" 2>&1; then
    echo "[FAIL] runtime error: $rel" >&2
    sed 's/^/    /' "$tmp_out" >&2 || true
    FAIL=1
    rm -f "$tmp_out"
    continue
  fi
  if [[ ! -f $expected ]]; then
     echo "[MISS] expected file not found: $expected" >&2
     FAIL=1
     rm -f "$tmp_out"
     continue
  fi
  # Normalize by stripping runtime warning lines to allow optional warnings.
  norm_actual="$(mktemp)"; norm_expected="$(mktemp)"
  grep -v '^\[TONG\]\[warn\]' "$tmp_out" > "$norm_actual" || true
  grep -v '^\[TONG\]\[warn\]' "$expected" > "$norm_expected" || true
  if diff -u --strip-trailing-cr "$norm_expected" "$norm_actual" > /dev/null; then
  echo "[PASS] $rel"
  (( ++PASS ))
  else
   echo "[DIFF] $rel" >&2
   echo '--- Raw diff including warnings (expected vs actual) ---' >&2
   diff -u --strip-trailing-cr "$expected" "$tmp_out" || true
   echo '--- Normalized diff (warnings stripped) ---' >&2
   diff -u --strip-trailing-cr "$norm_expected" "$norm_actual" || true
   FAIL=1
  fi
  rm -f "$tmp_out"
  rm -f "$norm_actual" "$norm_expected"
done

echo "== Summary =="
echo "Total: $TOTAL  Passed: $PASS  Failed: $(( TOTAL-PASS ))"
if (( FAIL )); then
  echo "[RESULT] FAIL" >&2
  exit 1
else
  echo "[RESULT] OK"
fi
