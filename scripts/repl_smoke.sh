#!/usr/bin/env bash
# REPL smoke test for core language features.
# Usage:
#   bash scripts/repl_smoke.sh          # compare against snapshot
#   UPDATE=1 bash scripts/repl_smoke.sh # (re)generate snapshot
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"
BIN="cargo run --quiet --manifest-path rust/tong/Cargo.toml --"
SNAP="examples/expected/repl_smoke.out"
TMP="$(mktemp)"

feed() {
  # All commands end with :quit to exit cleanly.
  cat <<'EOF'
let x = 10
let y = 32
print("sum", x + y)

# data + pattern functions
data Maybe = Nothing | Just v
def fromMaybe(Just(v)) { v }
def fromMaybe(Nothing) { 0 }
print("fmJ", fromMaybe(Just(5)))
print("fmN", fromMaybe(Nothing))

# guarded factorial
def fact(0) { 1 }
def fact(n) if n > 0 { n * fact(n - 1) }
print("fact5", fact(5))

# lambdas (backslash + pipe) and partials
let inc = \a -> a + 1
let sq = |n| n * n
print("inc41", inc(41))
print("sq7", sq(7))
let add = \a b -> a + b
print("add2_3", add(2,3))

# list comprehensions (single + multi) + predicate
let nums = [1,2,3,4,5]
print("squares", [ x*x | x in nums ])
print("pairs", [ (x,y) | x in nums, y in nums if x < y & x + y < 7 ])

# logical operators & || ! and short-circuit demo
let side = [0]
let short1 = false & (side[0] = side[0] + 1)
let tmpS = side[0]
print("short1", short1, tmpS)
let short2 = true || (side[0] = side[0] + 1)
print("short2", short2, side[0])
let short3 = !false & true || false
print("logicMix", short3)

# array element update sugar
let arr = [0,1,2]
arr[1] = arr[1] + 10
print("arr", arr[0], arr[1], arr[2])

# match with guard
match Just(42) { Just(v) if v > 40 -> print("matchJ", v), Nothing -> print("matchN") }

:quit
EOF
}

# Run REPL with fed script
if ! feed | eval "$BIN" > "$TMP" 2>&1; then
  echo "[REPL] execution failed" >&2
  sed 's/^/  | /' "$TMP" >&2 || true
  rm -f "$TMP"
  exit 1
fi

if [[ "${UPDATE:-0}" == "1" ]]; then
  mkdir -p "$(dirname "$SNAP")"
  cp "$TMP" "$SNAP"
  echo "[repl] snapshot (re)generated at $SNAP"
  rm -f "$TMP"
  exit 0
fi

if [[ ! -f $SNAP ]]; then
  echo "[repl] missing snapshot: $SNAP (run with UPDATE=1)" >&2
  rm -f "$TMP"
  exit 1
fi

if diff -u --strip-trailing-cr "$SNAP" "$TMP" > /dev/null; then
  echo "[repl] PASS"
  rm -f "$TMP"
  exit 0
else
  echo "[repl] DIFF" >&2
  diff -u --strip-trailing-cr "$SNAP" "$TMP" || true
  rm -f "$TMP"
  exit 1
fi
