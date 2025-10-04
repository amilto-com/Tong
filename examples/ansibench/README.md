# Ansibench examples for Tong

This folder contains ports of classic micro-benchmarks.

## Whetstone flags

Whetstone accepts a few simple CLI flags and values:

- Continuous mode:
  - `-c` — run repeatedly until interrupted
- Bounded repeats (number of runs):
  - `-n N` — run N times, then exit
  - key=value: `iterations=N` or `n=N`
- Loop count override (work size):
  - positional: `<loops>` (any positive integer)
  - key=value: `loops=<N>`

Examples:

- Single run with loop override:
  - `tong examples/ansibench/whetstone.tong 7`
- Run 3 times (no continuous):
  - `tong examples/ansibench/whetstone.tong -n 3`
- Continuous for exactly 3 runs, then exit:
  - `tong examples/ansibench/whetstone.tong -c -n 3 6`
- Key=value forms:
  - `tong examples/ansibench/whetstone.tong loops=1000 iterations=2`

Notes:
- If multiple values are provided, later positional integers can override earlier key=value forms for loops.
- Non-positive or non-numeric values are ignored.
