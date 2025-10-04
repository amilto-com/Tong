# The Computer Language Benchmarks Game (Tong)

This directory contains Tong implementations of benchmarks from:
https://benchmarksgame-team.pages.debian.net/benchmarksgame/

Goals:
- Faithful, fast implementations with simple CLI to match the benchmark harnesses.
- One `.tong` file per benchmark. Prefer pure Tong + std builtins; avoid external modules.

Status
- Initial drop includes compute-only kernels; more to come.

Conventions
- Each program accepts the standard single integer argument `N` where applicable.
- Print formats follow the benchmarksgame reference outputs.

Notes
- Some tasks (e.g., regex-redux, pidigits) may need additional runtime features for peak speed.
- We’ll iterate to optimize hot loops, reduce allocations, and exploit in-place updates.
