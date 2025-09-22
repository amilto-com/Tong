# Rosetta Code examples in TONG

This folder contains implementations of popular Rosetta Code tasks in TONG’s current syntax/features.

Included tasks:
- Hello world
- FizzBuzz
- Fibonacci (iterative)
- Greatest common divisor (Euclid)
- Factorial (iterative)
- Collatz sequence
- Prime factors
- 100 doors
- N-body problem (3-body, velocity Verlet)
- Towers of Hanoi

Run any example, e.g.:

```
python tong.py examples/rosetta/fizzbuzz.tong
```

Notes:
- Arrays are currently literal-constructed; mutation is limited to index assignments like `a[i] = value`.
- Lambdas and higher-order functions are supported. Parallel constructs in comments are illustrative.
- The N-body example uses velocity Verlet for stability.