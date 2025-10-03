#!/usr/bin/env bash
# Comprehensive regression checker for all non-SDL examples.
# Usage: bash scripts/check_examples.sh
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

FAIL=0
TOTAL=0
PASS=0
UPDATED=0

FILES=()

# Focused mode: if $FILES env var is provided, only run those (comma or space separated)
if [[ -n "${FILES:-}" ]]; then
  IFS=', ' read -r -a __req <<< "${FILES}"
  for item in "${__req[@]}"; do
    [[ -z $item ]] && continue
    # Normalize path
    if [[ -f $item ]]; then
      cand="$item"
    elif [[ -f examples/$item ]]; then
      cand="examples/$item"
    else
      echo "[SKIP] not found: $item" >&2
      continue
    fi
    if [[ $cand != examples/* ]]; then
      echo "[SKIP] outside examples/: $cand" >&2
      continue
    fi
    FILES+=("$cand")
  done
else
  while IFS= read -r -d '' f; do
    FILES+=("$f")
  done < <(find examples -type f -name '*.tong' -print0 | sort -z)
fi

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
  # continue handled above; leave counters intact
  fi
  if [[ ! -f $expected ]]; then
       if [[ "${UPDATE:-0}" == "1" ]]; then
          mkdir -p "$(dirname "$expected")"
          cp "$tmp_out" "$expected"
          echo "[CREATE] $rel (new snapshot)"
          (( ++PASS ))
          (( ++UPDATED ))
          rm -f "$tmp_out"
          continue
       else
          echo "[MISS] expected file not found: $expected" >&2
          FAIL=1
          rm -f "$tmp_out"
          continue
       fi
  fi
    if diff -u --strip-trailing-cr "$expected" "$tmp_out" > /dev/null; then
      echo "[PASS] $rel"
      (( ++PASS ))
    else
      if [[ "${UPDATE:-0}" == "1" ]]; then
        cp "$tmp_out" "$expected"
        echo "[UPDATE] $rel"
        (( ++PASS ))
        (( ++UPDATED ))
      else
        echo "[DIFF] $rel" >&2
        diff -u --strip-trailing-cr "$expected" "$tmp_out" || true
        FAIL=1
      fi
    fi
  rm -f "$tmp_out"
done

echo "== Summary =="
echo "Total: $TOTAL  Passed: $PASS  Failed: $(( TOTAL-PASS ))  Updated: $UPDATED"
if (( FAIL )); then
  echo "[RESULT] FAIL" >&2
  exit 1
else
  if (( UPDATED > 0 )); then
    echo "[RESULT] OK (updated $UPDATED snapshots)"
  else
    echo "[RESULT] OK"
  fi
fi
