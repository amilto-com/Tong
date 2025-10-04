# Tong Language User Guide

This user guide is an exhaustive, practical reference for the Tong language as implemented in this repository (Rust MVP). It covers syntax, semantics, standard operators, control flow, functions, pattern matching, data types, modules, built‑ins, and the command‑line interface.

If you’re new, skim Quick Start first, then use the rest as a reference.

---

## Table of contents

- [Quick start](#quick-start)
- [Language overview](#language-overview)
- [Lexical elements](#lexical-elements)
- [Types and values](#types-and-values)
- [Expressions and operators](#expressions-and-operators)
- [Statements](#statements)
- [Functions and lambdas](#functions-and-lambdas)
- [Pattern functions and match](#pattern-functions-and-match)
- [Arrays and comprehensions](#arrays-and-comprehensions)
- [Control flow](#control-flow)
- [Built‑in functions (core)](#built-in-functions-core)
- [Modules](#modules)
  - [args module](#args-module)
  - [sdl module (shim in default build)](#sdl-module-shim-in-default-build)
  - [linalg module](#linalg-module)
- [Command line interface](#command-line-interface)
- [REPL](#repl)
- [Standard library idioms](#standard-library-idioms)
- [Error handling and diagnostics](#error-handling-and-diagnostics)
- [Gotchas and tips](#gotchas-and-tips)
 - [Gradual typing (optional annotations)](#gradual-typing-optional-annotations)

---
 - Typed examples: see `examples/typed/` and the dedicated doc `doc/typing_guide.md` for syntax and runtime behavior.
## Quick start
## Gradual typing (optional annotations)

Tong supports optional type annotations on let/var bindings and on simple function parameters and return types.

- Binding annotations: `let x: Int = 42`, `var name: Str = "Mozza"`, `let xs: Array = [1,2]`, `let any: Any = f()`
- Function annotations (simple parameters only): `fn add(a: Int, b: Int) -> Int { return a + b }`
- Guarded functions: `fn f(x: Int) -> Int if x > 0 { ... }` (params + optional return)
- Pattern functions: return annotation supported: `def head(xs) -> Int { ... }`
- Enforced at runtime for now: arguments and returns are checked when annotated; bindings are checked on assignment.
- A small static lint pass warns on obvious literal mismatches before running.
- Pattern params don’t support per-parameter annotations yet; guarded functions do for simple params. Pattern and guarded functions both support an optional return annotation.

Types available: `Int`, `Float`, `Bool`, `Str`, `Array`, `Any` (disables checking).

- Run a file
  - tong path/to/file.tong [script-args...]
- Start REPL
  - tong
- List built‑ins and modules
  - tong --list-builtins
  - tong --modules

Minimal program:

```
fn main {
  print("Hello, Tong!")
}
```

---

## Language overview

- Expression‑first with blocks that can return values
- Static surface grammar with a small set of keywords
- Functions are first‑class and support anonymous forms
- Pattern functions (by constructor and tuples) and guards
- Arrays as the primary aggregate collection
- Basic concurrency with `parallel { ... }` block
- Optional SDL module (headless shim by default unless built with sdl3 feature)

---

## Lexical elements

- Identifiers: [A-Za-z_][A-Za-z0-9_]*
- Literals:
  - Strings: "..." (no escapes yet in MVP)
  - Integers: 0, 1, 2, ...
  - Floats: 0.5, 3.14 (digits.digits)
  - Booleans: true, false
- Comments: // to end of line

---

## Types and values

Primitive values:
- Int (signed 64‑bit)
- Float (64‑bit)
- Bool
- Str
- Array[TongValue]
- Function/Lambda
- Objects (module instances and constructed records)

Pattern/data modeling:
- Algebraic data declarations via `data` with constructors
  - Example: `data Option = None | Some a`
    - In MVP, constructor arity is determined by parameter count; generic type vars are lexical only.

---

## Expressions and operators

Precedence (low to high):
1. Logical OR: `||`
2. Logical AND: `&&`
3. Bitwise OR: `|`
4. Bitwise XOR: `^`
5. Bitwise AND: `&`
6. Shifts: `<<`, `>>`
7. Comparisons: `<`, `<=`, `>`, `>=`
8. Equality: `==`, `!=`
9. Additive: `+`, `-`
10. Multiplicative: `*`, `/`, `%`
11. Unary: `-x`, `!x`, `+x`
12. Primary: literals, identifiers, calls, indexing, property/method, parenthesized, block

Notes:
- `&&` and `||` are logical short‑circuit operators.
- `&`, `|`, `^`, `<<`, `>>` are bitwise for Int; for Bool, `&`, `|`, `^` are supported as non‑short‑circuit logical operations.
- Right shift semantics: arithmetic vs logical are implementation details; treat as shift of 64‑bit signed integer with well‑defined behavior for non‑negative values.

Array and object access:
- Indexing: `arr[i]`
- Property: `obj.prop`
- Method call: `obj.method(arg1, arg2)`

Blocks:
- `{ stmt* }` is an expression; returns the last expression value in the block if present.

---

## Statements

- `let name = expr`
- `let (a,b,c) = expr` (tuple destructuring over arrays)
- `name[index] = expr` (array element assignment)
- `print(expr1, expr2, ...)`
- `if cond { ... } else { ... }` (block form; last expr value is returned from block)
- `while cond { ... }`
- `parallel { stmt* }` (MVP: executes sequentially; future may parallelize)
- `return expr` inside functions
- `data Type = Ctor ... | Ctor ...` (declares constructors)
- Function definitions:
  - `fn main { ... }` entry point
  - `fn name(a, b) { ... }` simple parameters
  - Guarded: `fn f(x) if x > 0 { ... }`
  - Pattern parameters: `def f(Some x) { ... }` or tuple‑like patterns

---

## Functions and lambdas

Named functions:
```
fn add(a, b) {
  a + b
}
```
Anonymous functions:
- Keyword lambda: `fn x y { x + y }`
- Backslash lambda: `\x y -> x + y`
- Pipe lambda: `|x| x + 1`

Calls:
- `add(1, 2)`
- Method style: `obj.method(arg)`

Returns:
- `return expr` or implicit last expression of the block.

---

## Pattern functions and match

Pattern parameters:
```
data List = Nil | Cons a List

def head(Cons x _){ x }
```

Guards:
```
def sign(x) if x > 0 { 1 }
```

Match expression:
```
match xs {
  Nil        -> 0,
  Cons x _   -> x
}
```

Notes:
- Wildcard is `_`.
- Constructor detection prefers semantic knowledge from `data`; capitalization heuristic is a fallback.

---

## Arrays and comprehensions

Arrays:
```
let xs = [1,2,3]
let x = xs[0]
xs[1] = 42
```

List comprehension:
```
let ys = [ x*x | x in xs ]
let evens = [ x | x in xs if x % 2 == 0 ]
```

---

## Control flow

- If statement uses blocks:
```
if cond {
  ...
} else {
  ...
}
```

- While loop:
```
while i < n {
  i = i + 1
}
```

- Parallel block:
```
parallel {
  a = f()
  b = g()
}
```

---

## Built‑in functions (core)

Core built‑ins available via direct call names:
- print(x, ...): prints values separated by space
- len(arr | str): length
- sum(arr[Int | Float]): numeric sum
- filter(arr, fn): returns new array with items where fn(item) is true
- map(arr, fn): returns new array with fn(item) results
- reduce(arr, fn, init): left fold; fn(acc, item) -> acc
- range(n) or range(start, end[, step]): integer sequence as array
- repeat(value, count): array repeating value
- sqrt(x), sin(x), cos(x), exp(x), log(x), abs(x)
- now_ms(): Int current time in milliseconds
- sleep_ms(ms): pause current thread
- import("module"): import a built‑in module, returns an object value
- getenv("NAME"): returns environment variable value or empty string

Notes:
- Functions accept intuitive types; operations on arrays are eager.

---

## Modules

Import modules with `let m = import("name")` and then use properties/methods on `m`.

### args module

Purpose: access script arguments passed after the file name on the `tong` CLI.

- Properties:
  - `script: String` (the file path, or empty if REPL)
  - `args: Array[String]` (arguments only)
  - `all: Array[String]` (script + arguments)
- Methods:
  - `len(): Int` — number of arguments
  - `get(i: Int): String` — positional argument at index i, or "" if out of range
  - `has(flag: String): Bool` — true if `flag` exactly appears in args
  - `value(key: String): String` — supports `--key value` and `--key=value`; returns "" if not present

Example:
```
let args = import("args")
print("script:", args.script)
print("argc:", args.len())
print("all:", args.all)
print("has --verbose:", args.has("--verbose"))
print("mode:", args.value("--mode"))
```

CLI usage supports flags without needing `--` separator, e.g.:
- tong examples/args_demo.tong --verbose --mode=fast abc foo

### sdl module (shim in default build)

- When built without `sdl3` feature: a headless shim is provided; real graphics calls are no‑ops that log a one‑time notice.
- When built with `sdl3` feature: provides typical SDL window/input/render hooks. See `examples/modules/sdl` for usage.

### linalg module

- Provides simple linear algebra/tensor utilities for numerical examples.
- See examples for capabilities.

---

## Command line interface

Usage:
- Run script: tong <FILE> [SCRIPT_ARGS]...
- REPL: tong
- Helpful flags:
  - --modules           List built‑in modules
  - --list-builtins     List core built‑in functions
  - -d, --debug         Verbose runtime tracing
  - --version           Short version (cargo pkg)
  - --version-long      Extended version (git hash, ts)
  - --show-exit         Print explicit exit status line

Script arguments:
- All tokens after <FILE> are captured verbatim (including hyphenated) and passed to the args module.
- A `--` separator is optional; if present it will appear inside `args.all` and `args.args` as a separate item.

---

## REPL

- Start: `tong`
- Special commands:
  - :help     Show help
  - :env      List variables
  - :modules  List built‑in modules
  - :reset    Clear state
  - :quit/:q/:exit Exit
- Multi‑line blocks are supported; prompt changes to `....` while a block is open.

---

## Standard library idioms

Map/filter/reduce:
```
let xs = [1,2,3,4]
let squares = map(xs, |x| x*x)
let odds = filter(xs, |x| x % 2 == 1)
let total = reduce(xs, fn acc x { acc + x }, 0)
```

Ranges and repeats:
```
let a = range(5)            // [0,1,2,3,4]
let b = range(2, 10, 2)     // [2,4,6,8]
let z = repeat(42, 3)       // [42,42,42]
```

Timing:
```
let t0 = now_ms()
sleep_ms(50)
let dt = now_ms() - t0
print("slept:", dt, "ms")
```

Bitwise:
```
let x = (a ^ b) & 65535
let y = x << 3
let z = (x >> 2) | 1
```

---

## Error handling and diagnostics

- Parser errors report line and column with nearby token.
- Runtime errors (e.g., undefined variable, index out of bounds) abort execution with an error message.
- Enable `--debug` for verbose trace of top‑level statements and certain runtime events.

---

## Gotchas and tips

- If expressions must use blocks; inline ternary‑style is not supported. Use:
```
let n = {
  if cond {
    1
  } else {
    2
  }
}
```
- Use `&&`/`||` for logical ops; `&`/`|` are bitwise (but also work on Bool without short‑circuit).
- Strings have no escape sequences yet; prefer simple ASCII or split/concat.
- Arrays are 0‑based; out‑of‑bounds raises an error.
- Constructors are recognized by prior `data` declarations or Capitalized identifiers.

---

## Examples

See the `examples/` folder for a broad set: basics, arrays, functions, modules, and ansibench variants (Dhrystone, NBench, CoreMark‑like, STREAM, Whetstone, LINPACK‑like, HINT‑like). Many benchmark examples accept `TONG_MODE=quick|full` via environment or use the args module.

---

## Version and build

- `--version`: shows cargo package version
- `--version-long`: shows git hash, dirty status, and build timestamp if provided at build time
- Building with `sdl3` feature enables real SDL bindings; otherwise a headless shim is used.

---

## Roadmap highlights

- String escapes and slices
- Dictionaries/records
- Modules as packages
- Concurrency primitives beyond parallel block
- Improved pattern match diagnostics

If you find discrepancies between the guide and the implementation, please open an issue or PR.
