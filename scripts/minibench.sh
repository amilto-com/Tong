#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR/examples/minibench"

# Build C benchmarks
make -s

run_one() {
  local name="$1"; shift
  local N="${1:-}"
  echo "== $name =="
  if [[ -n "$N" ]]; then
    "$ROOT_DIR"/rust/tong/target/release/tong "$ROOT_DIR/examples/minibench/${name}.tong" "$N"
    ./"$name" "$N"
  else
    "$ROOT_DIR"/rust/tong/target/release/tong "$ROOT_DIR/examples/minibench/${name}.tong"
    ./"$name"
  fi
}

# Allow selecting a single bench
if [[ $# -ge 1 ]]; then
  run_one "$1" "${2:-}"
  exit 0
fi

# Otherwise run all with defaults
for b in sum_loop sum_float fib_iter fib_rec array_fill array_sum bitwise call_overhead branch math_sin; do
  run_one "$b"
  echo
done
