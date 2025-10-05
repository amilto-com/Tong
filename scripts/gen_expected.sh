#!/usr/bin/env bash
# Generate expected outputs for all non-SDL examples.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
cd "$ROOT_DIR"

out_dir=examples/expected
mkdir -p "$out_dir"

# Function: run one example and store output
run_example() {
  local src="$1" rel
  rel="${src#examples/}"
  rel="${rel%.tong}"
  local target="$out_dir/${rel}.out"
  mkdir -p "$(dirname "$target")"
  echo "[gen] $src -> $target"
  cargo run --quiet --manifest-path rust/tong/Cargo.toml -- "$src" > "$target"
}

# Iterate examples (skip SDL module examples)
while IFS= read -r -d '' f; do
  if [[ $f == examples/modules/sdl/* ]]; then
    continue
  fi
  run_example "$f"
# Use -print0 to be safe with spaces (none expected, but future proof)
done < <(find examples -type f -name '*.tong' -print0 | sort -z)

echo "[gen] Done."
