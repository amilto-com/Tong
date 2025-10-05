Mini micro-benchmarks (C vs Tong)

This folder contains 10 ultra-simple micro-benchmarks implemented in both C and Tong.
Each program accepts an optional single argument N controlling the problem size.

For Tong, each program measures its own elapsed time in milliseconds using now_ms().
For C, each program measures elapsed time using clock_gettime(CLOCK_MONOTONIC).

List of benchmarks:
- sum_loop: integer sum 1..N
- sum_float: floating-point accumulation
- fib_iter: iterative Fibonacci up to N
- fib_rec: naive recursive Fibonacci(N)
- array_fill: allocate/fill array with i
- array_sum: sum elements of a filled array
- bitwise: bitwise shifts/xor/and in a tight loop
- call_overhead: function call overhead in a loop
- branch: branching and counters
- math_sin: accumulate sin(i) over a loop

How to run (examples):
- Tong: rust/tong/target/release/tong examples/minibench/sum_loop.tong 1000000
- C build+run: gcc -O3 -march=native -pipe -std=c11 -lm -o /tmp/sum_loop examples/minibench/sum_loop.c && /tmp/sum_loop 1000000

Notes:
- Defaults are chosen to complete quickly on Tong; pass larger N to stress.
- Tong prints both result and time_ms for convenience.
