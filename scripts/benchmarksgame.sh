#!/usr/bin/env bash
# Helper to run a subset of the Computer Language Benchmarks Game programs in Tong
# Usage: bash scripts/benchmarksgame.sh [program] [N]
#   program: binarytrees | spectralnorm | nbody | fannkuch-redux | fasta | mandelbrot
#   N: integer problem size (default varies per program)
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

prog="${1:-}"
N="${2:-}"
file=""
case "$prog" in
  binarytrees) file="examples/benchmarksgame/binarytrees.tong" ;;
  spectralnorm) file="examples/benchmarksgame/spectralnorm.tong" ;;
  nbody) file="examples/benchmarksgame/nbody.tong" ;;
  fannkuch-redux) file="examples/benchmarksgame/fannkuch-redux.tong" ;;
  fasta) file="examples/benchmarksgame/fasta.tong" ;;
  mandelbrot) file="examples/benchmarksgame/mandelbrot.tong" ;;
  "") echo "Usage: $0 <program> [N]"; exit 1 ;;
  *) echo "Unknown program: $prog"; exit 1 ;;
esac

if [[ -n "$N" ]]; then
  cargo run --quiet --manifest-path rust/tong/Cargo.toml -- "$file" "$N"
else
  cargo run --quiet --manifest-path rust/tong/Cargo.toml -- "$file"
fi
