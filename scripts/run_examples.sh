#!/usr/bin/env bash
# Run all top-level .tong examples (excluding module subdirectories) using the Rust tong CLI.
# Usage:
#   ./scripts/run_examples.sh            # auto-detect (prefers release, then debug, then build)
#   TONG=path/to/tong ./scripts/run_examples.sh
#   ./scripts/run_examples.sh --all      # include module examples (examples/modules/*/*.tong)
#   ./scripts/run_examples.sh --rosetta  # include rosetta examples only
#
set -euo pipefail
shopt -s nullglob

include_modules=false
only_rosetta=false
skip_sdl=false
only_ansibench=false
mode=""

for arg in "$@"; do
  case "$arg" in
  --all) include_modules=true ;;
  --rosetta) only_rosetta=true ;;
  --ansibench) only_ansibench=true ;;
  --skip-sdl) skip_sdl=true ;;
    -h|--help)
      cat <<EOF
Run TONG examples.

Options:
  --all        Include module examples under examples/modules/**
  --rosetta    Run only Rosetta examples (examples/rosetta/*.tong)
  --ansibench  Run only Ansibench examples (examples/ansibench/*.tong)
  --skip-sdl   Skip examples under examples/modules/sdl (avoid window launch)
  --quick      Set TONG_MODE=quick to run smaller problem sizes
  --full       Set TONG_MODE=full to run larger/standard sizes
  -h, --help   Show this help

Environment:
  TONG   Path to tong executable (if unset the script will try to locate or build one)
EOF
      exit 0
      ;;
    --quick) mode="quick" ;;
    --full) mode="full" ;;
    *) echo "Unknown option: $arg" >&2; exit 1 ;;
  esac
done

# Resolve tong executable
if [[ -z "${TONG:-}" ]]; then
  root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
  # Candidate paths
  candidates=(
    "$root_dir/rust/tong/target/release/tong"
    "$root_dir/rust/tong/target/debug/tong"
    "$HOME/.cargo/bin/tong"
  )
  for c in "${candidates[@]}"; do
    if [[ -x "$c" ]]; then TONG="$c"; break; fi
  done
fi

if [[ -z "${TONG:-}" ]]; then
  echo "[info] Building tong (debug)..." >&2
  (cd "$(dirname "${BASH_SOURCE[0]}")/../rust/tong" && cargo build >/dev/null)
  TONG="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/rust/tong/target/debug/tong"
fi

if [[ ! -x "$TONG" ]]; then
  echo "[error] Could not find or build 'tong' executable (looked at $TONG)" >&2
  exit 1
fi

echo "[using] tong executable: $TONG"
if [[ -n "$mode" ]]; then
  export TONG_MODE="$mode"
  echo "[mode] TONG_MODE=$TONG_MODE"
fi

examples_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../examples" && pwd)"

declare -a files
if $only_rosetta; then
  while IFS= read -r -d '' f; do files+=("$f"); done < <(find "$examples_root/rosetta" -maxdepth 1 -type f -name '*.tong' -print0 | sort -z)
elif $only_ansibench; then
  if [[ -d "$examples_root/ansibench" ]]; then
    while IFS= read -r -d '' f; do files+=("$f"); done < <(find "$examples_root/ansibench" -maxdepth 1 -type f -name '*.tong' -print0 | sort -z)
  fi
else
  # Top-level examples first (exclude nested directories)
  while IFS= read -r -d '' f; do files+=("$f"); done < <(find "$examples_root" -maxdepth 1 -type f -name '*.tong' -print0 | sort -z)
  if $include_modules; then
    while IFS= read -r -d '' f; do files+=("$f"); done < <(find "$examples_root/modules" -type f -name '*.tong' -print0 | sort -z || true)
  fi
  # Always include rosetta unless only_rosetta is set
  while IFS= read -r -d '' f; do files+=("$f"); done < <(find "$examples_root/rosetta" -maxdepth 1 -type f -name '*.tong' -print0 | sort -z)
fi

if [[ ${#files[@]} -eq 0 ]]; then
  echo "[warn] No example .tong files found." >&2
  exit 0
fi

for f in "${files[@]}"; do
  rel="${f#$examples_root/}"
  echo -e "\n=== Running $rel ==="
  if $skip_sdl && [[ "$rel" == modules/sdl/* ]]; then
    echo "[skip] SDL example skipped due to --skip-sdl"
    continue
  fi
  if ! "$TONG" "$f"; then
    echo "[fail] Example failed: $rel" >&2
    exit 1
  fi
done

echo -e "\nAll selected examples completed successfully."