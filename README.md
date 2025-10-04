# TONG Programming Language

[![CI](https://github.com/amilto-com/Tong/actions/workflows/ci.yml/badge.svg)](https://github.com/amilto-com/Tong/actions/workflows/ci.yml)

<p align="center">
    <img src="images/TONG_a35c8343-4727-4443-bdaf-7ab3e9d3661d.jpg" alt="TONG Logo" width="320" />
</p>

The ultimate programming language designed for high‑performance parallel and distributed computing across heterogeneous environments (CPU/GPU/NPU/FPGA).

Authored by William Gacquer — AMILTO

Contributions welcome! See “Contributing” below.

## User Guide

Looking for the language reference and practical how‑tos? Read the comprehensive Tong User Guide:

- doc/user_guide/tong_user_guide.md

Gradual typing (optional annotations) overview:

- doc/typing_guide.md
- See annotated examples in `examples/typed/`.

## Why the name “TONG”?

We wanted a name that’s short, fun, and nerd‑accurate:

- It sounds like “tongue,” which you use for speaking a language — perfect for a programming language. Linguists, you’re welcome.
- In French, “une bascule (T flip‑flop)” is colloquially referred to as a “T(ong)” style flip‑flop — the simplest building block of memory in digital hardware. TONG pays homage to that first tiny bit of “remembering” your computer ever did.
- It’s easy to say, hard to forget, and looks great in monospace.

Conclusion: TONG is a language that speaks performance and remembers what matters.

## Features

🚀 **High Performance**
- Zero-cost abstractions
- Automatic parallelization for large datasets
- Memory safety without garbage collection overhead
- JIT compilation in REPL mode

⚡ **Heterogeneous Computing**
- Clear path toward CPU/GPU/NPU/FPGA execution
- Automatic workload distribution (progressive rollout)
- GPU kernel compilation (design underway)
- Distributed computing primitives (incremental)

🔧 **Developer Experience**
- Interactive REPL with hot compilation
- Modern syntax combining best of all languages
- Comprehensive error messages
- Built-in parallel algorithms

🌐 **Compilation Targets**
- Interpreter today, with compilation paths on the roadmap:
    - Native (x86_64/ARM64/RISC‑V)
    - WebAssembly (WASM)
    - GPU shaders (CUDA/OpenCL/Metal)
    - FPGA HDL

## Quick Start

### Installation

Clone the repository:

```bash
git clone https://github.com/amilto-com/Tong.git
cd Tong
```

Install Rust (if you don't have it): https://rustup.rs

Optional: add a convenient “tong” command to your PATH

- macOS/Linux:
    ```bash
    ./setup.sh --global
    ```
- Windows (PowerShell, no admin required):
    ```powershell
    ./setup.ps1 -Global
    ```

### Running TONG

```bash
# From the Rust crate directory
cd rust/tong

# Run a TONG program
cargo run -- ../../examples/hello.tong

# Build a release binary
cargo build --release

# After running setup.sh --global or setup.ps1 -Global
tong ../../examples/hello.tong
 
Windows (SDL3 example):
- The SDL3 backend is optional. To run the SDL Pong example with a real window, build with the feature and ensure `SDL3.dll` is available at runtime.
- Recommended options:
    - Place `SDL3.dll` next to the built binary (e.g., `rust/tong/target/debug/`), or
    - Put `SDL3.dll` in a directory on your PATH.
- We do not commit `SDL3.dll` to the repository; see `.gitignore`.

Example:
```powershell
# Build with SDL3 backend
cargo build --features sdl3

# Run the Pong example (ensure SDL3.dll is discoverable)
.\target\debug\tong.exe ..\..\examples\modules\sdl\pong.tong
```

### Linux / WSL (SDL3 example)

If you run `pong.tong` and NO window appears, you almost certainly built without the `sdl3` feature. Without that feature, the runtime uses a headless shim (no real window) so CI/tests can pass. Re‑build with the feature enabled.

```bash
cd rust/tong
# Debug run
cargo run --features sdl3 -- ../../examples/modules/sdl/pong.tong

# Or build release
cargo build --release --features sdl3
../../target/release/tong ../../examples/modules/sdl/pong.tong
```

#### WSL specifics

WSL2 with WSLg (Windows 11, or updated Windows 10) supports Wayland/X11 out of the box. Just enabling the feature is usually enough. Verify GUI support with:

```bash
echo $WAYLAND_DISPLAY  # should be non-empty on WSLg
```

If you are on older WSL1 (or WSL2 without WSLg) you need an external Windows X server (VcXsrv/Xming) and to export DISPLAY, e.g.:

```bash
export DISPLAY=$(ip route | awk '/default/ {print $3}'):0
export LIBGL_ALWAYS_INDIRECT=1   # sometimes needed for legacy setups
```

Then run the program again with the `sdl3` feature.

#### Dependencies for building SDL3 from source

The crate enables `features = ["build-from-source"]` for `sdl3`, so it will compile SDL3 locally. Make sure you have build tools and common video/audio dev packages:

```bash
sudo apt update
sudo apt install -y build-essential cmake ninja-build pkg-config \
    libwayland-dev libx11-dev libxext-dev libxrandr-dev libxinerama-dev \
    libxcursor-dev libxi-dev libdrm-dev libgbm-dev libpulse-dev
```

You can start with fewer packages; these cover most headless → window build failures. If the build still fails, check the first missing library mentioned by the SDL build logs.

#### Troubleshooting

| Symptom | Likely Cause | Fix |
|---------|--------------|-----|
| No window, program exits after a moment | Forgot `--features sdl3` | Re-run with feature |
| Build error about missing C compiler | No build tools | `sudo apt install build-essential` |
| Runtime: Cannot open display | WSL1 without X server | Install & start VcXsrv, set DISPLAY |
| Black window only | Rendering loop running but drawing state stuck | Ensure `present` is called (it is in example); try `SDL_VIDEODRIVER=x11` or `wayland` |

You can force a specific backend:

```bash
SDL_VIDEODRIVER=wayland cargo run --features sdl3 -- ../../examples/modules/sdl/pong.tong
# or
SDL_VIDEODRIVER=x11 cargo run --features sdl3 -- ../../examples/modules/sdl/pong.tong
```

If all else fails, run with `RUST_LOG=debug` after adding some debug prints (or temporarily instrument `runtime.rs`) to confirm the SDL path is actually compiled (look for `#[cfg(feature = "sdl3")]`).


## Language Examples

### Hello World
```tong
fn main() {
    print("Hello, TONG World!")
    print("The ultimate programming language is here!")
}

main()
```

### Variables and Functions
```tong
let x = 42
let name = "TONG"

// Functions with type inference
fn add(a, b) {
    a + b
}

fn factorial(n) {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}
```

### Arrays and Built-in Functions
```tong
let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

// Built-in functions
print("Sum:", sum(numbers))
print("Length:", len(numbers))
print("Squared:", map(numbers, square))
```

### Parallel Computing (today and tomorrow)
```tong
// Explicit parallel blocks
parallel {
    let result1 = heavy_computation1()
    let result2 = heavy_computation2()
    combine(result1, result2)
}

// GPU kernels (syntax design)
gpu_kernel fn matrix_multiply(a, b) {
    // Automatic GPU code generation
    a * b
}

// Distributed computing (syntax design)
distributed fn process_big_data(data) {
    data.parallel_reduce(|a, b| a + b)
}
```

### Functional / Phase 1 Additions (Experimental)

Early functional features inspired by Haskell (syntax may evolve):

| Feature | Example | Notes |
|---------|---------|-------|
| Algebraic data types | `data Maybe = Nothing | Just x` | Registers constructors & arities globally |
| Pattern matching | `match m { Just(x) -> x, Nothing -> 0 }` | Arm guards: `Just(x) if x > 0 -> x` |
| Wildcard | `_` | Matches anything, no binding |
| Constructor subpatterns | `Just(x)`, `Node(l,r)` | Positional only, parenthesized form preferred |
| Nested constructor patterns | `match t { Node(Leaf(a), Leaf(b)) -> ... }` | Arbitrary nesting supported (dynamic) |
| List comprehension | `[ x*x | x in xs if x%2==0 ]` / `[ (x,y) | x in xs, y in ys if cond ]` | Multiple generators + optional predicate |
| Lambdas (pipe) | `|x| x + 1` | Single param shorthand |
| Lambdas (backslash) | `\x y -> x + y` | Multi-arg, supports partials |
| Partial application | `let add2 = add(2)` | Functions & lambdas; supports currying
| Partial constructor application | `let left = Node(Leaf(1))` then `left(Leaf(2))` | Produces `<partial:Name:n>` until saturated |
| Tuple destructuring | `let (a,b) = pair` | Source must be array-like same length |
| Tuple patterns in match | `match t { (x,_,z) -> ... }` | Fixed-size array (tuple) patterns |
| Guarded multi-clause functions | `fn fact(n) if n==0 {1}` / `fn fact(n) if n>0 { n*fact(n-1) }` | First passing guard executes |
| Pattern clause functions (constructor patterns in params) | `def fromMaybe(Just(x)) { x } / def fromMaybe(Nothing) { 0 }` | Sugar over internal pattern match, supports guards |
| Logical operators | `a & b`, `a || b`, `!a` | Short-circuit AND / OR, unary NOT |

See `examples/features/` for runnable demonstrations.

Current limitations:
* Dynamic only (no static type checking yet).
* No exhaustive / redundancy analysis for `match` (missing case triggers runtime error).
* Each pattern clause supports a single guard: `def f(Just(x)) if x > 10 { ... }`.
* Tuple patterns treat arrays as tuples; no variadic / rest patterns.
* Error diagnostics are still minimal (work in progress).

Recent improvements:
* Multi-generator list comprehensions.
* Parenthesized constructor calls & patterns (`Just(42)`, `Node(left,right)`).
* Partial application generalized to constructors.
* Nested constructor patterns (example: `nested_patterns.tong`).
* Implicit last-expression return in function bodies (you can omit `return` for final expression).
* Short-circuit logical operators `&`, `||`, and unary `!`.
* Array element update sugar `arr[i] = expr` (immutably rebuilds array with updated slot).
* Explicit block expressions `{ stmt* }` evaluate to the last expression value.
* First-class anonymous `fn` with block bodies `let f = fn a b { let c = a + b; c * 2 }`.
* Indexing expressions `arr[i]` and chaining with other postfix forms (`{ [1,2,3] }[0]`).

### Block Expressions

TONG supports block expressions as values: a brace-delimited sequence of statements that yields the value of its final bare expression (if any). This enables inline scoping and multi-step computations inside larger expressions.

Example:
```tong
let result = {
    let a = 10
    let b = a * 3
    b + 7   // last expression becomes the block value (37)
}
print(result)
```
Semantics:
* All `let` / assignment statements inside the block are scoped to the block.
* If the block ends with a bare `Expr` statement, its value is returned; otherwise the block yields an empty array `[]` for now (placeholder “unit”).
* Blocks compose with other postfix operators: `{ [100,200,300] }[1]` -> `200`.

Anonymous function literals (`fn ... { ... }`) reuse block expression semantics so multi-statement function bodies can be written inline without defining a named function.

### Indexing & Postfix Chaining

Indexing is an expression form with the highest precedence tier (alongside call/property). It can be chained arbitrarily:
```tong
let grid = [[1,2],[3,4]]
print(grid[1][0])      // 3
{ [10,20,30] }[2]      // 30 (inside larger expression if desired)
```
Rules:
* Index expression `target[index]` evaluates `target` then `index` (left-to-right) and expects an array / list.
* Out-of-bounds access triggers a runtime error.
* Indexing works on the result of any expression, including constructor calls, lambdas returning arrays, or block expressions.

Array update sugar (`arr[i] = expr`) is statement-level and distinct from expression indexing: it produces an updated array value immutably (clones underlying vector and writes the slot).

### Anonymous Function Syntax Summary

Three equivalent anonymous function literal forms are currently supported (all produce a first-class function value and support partial application):

| Form | Example | Arity inference | Notes |
|------|---------|-----------------|-------|
| Pipe single-param | `let inc = |x| x + 1` | 1 param | Shorthand for a single argument; body is expression following `|param|`. |
| Backslash multi-param | `let add = \a b -> a + b` | Count identifiers before `->` | Multiple params separated by spaces; right side is a single expression. |
| `fn` with block | `let f = fn a b { let c = a + b; c * 2 }` | Count identifiers before `{` | Full block body: any number of statements; last bare expression is return value. |

All forms support partial application (currying) when invoked with fewer arguments than declared.
Examples:
```tong
let add3 = \a b c -> a + b + c
let add1 = add3(1)       // <partial:add3:1>
let add1_2 = add1(2)     // <partial:add3:2>
print(add1_2(7))         // 10

let scale_then = fn factor { fn x { x * factor } }
let double = scale_then(2)
print(double(21))        // 42
```

### Operator Precedence (Highest → Lowest)

1. Indexing / Call / Property (postfix)
2. Unary: `-`, `+`, `!`
3. Multiplicative: `*`, `/`, `%`
4. Additive: `+`, `-`
5. Comparison: `<`, `<=`, `>`, `>=`
6. Equality: `==`, `!=`
7. Conjunction: `&`
8. Disjunction: `||`
9. (Assignments are statement-level only: `let x =`, `x =`, `arr[i] =`)

All binary operators are left-associative. Parentheses may be used to override default grouping.
* Short-circuit logical operators `&` (AND), `||` (OR) and unary `!` (NOT).
* Array element update sugar: `arr[i] = expr` (clones + updates underlying array immutably).

### Operator Precedence (highest → lowest)

The parser implements a conventional precedence ladder. Parentheses `(...)` may always be used to override defaults.

1. Indexing / Property / Call chaining: `arr[i]`, `obj.prop`, `func(x)` (left-associative)
2. Unary prefix: `!expr`, `-expr`, `+expr`
3. Multiplicative: `*`, `/`, `%`
4. Additive: `+`, `-`
5. Comparison: `<`, `<=`, `>`, `>=`
6. Equality: `==`, `!=`
7. Logical AND: `&` (short‑circuit)
8. Logical OR: `||` (short‑circuit)

Notes:
* All binary operators are left-associative currently.
* `&` and `||` short‑circuit: the right operand is only evaluated if needed.
* Unary `!` expects a Bool; unary `-` expects numeric.
* There is no assignment expression; `=` is only a statement form (`let x = ...` or `x = ...`).
* `arr[i] = v` is syntactic sugar for cloning the array and writing index `i`; bounds checked.
* Future additions (e.g. exponentiation) may introduce a new higher precedence tier.

Planned next:
* Additional guarded pattern clause examples (showing ordering & guard short‑circuit).
* Exhaustiveness & redundancy warnings.
* Type annotations & inference groundwork.
* Improved partial introspection / debug printing.
* Better error spans / diagnostics.

### Constructor Partial Application

Constructors behave like functions: applying fewer arguments than the declared arity produces a partial that can be saturated later.

Example (`examples/features/constructor_partial.tong`):
```tong
data Tree = Leaf v | Node left right

let left_only = Node(Leaf(1))      // <partial:Node:1>
let full = left_only(Leaf(2))      // Node(Leaf(1),Leaf(2))
print(full)
```
This mirrors partial application for functions and lambdas and enables ergonomic construction of deeply nested values.

## REPL
Run without a file to enter the interactive REPL:

```bash
cd rust/tong
cargo run --features sdl3 --   # or omit feature if you don't need graphics
```

Commands:

```
:help   Show help
:env    List current variables
:reset  Clear all user-defined variables and functions
:quit   Exit the REPL (:q / :exit also work)
```

CLI discovery:

```bash
tong --modules   # list built-in modules (e.g. sdl, linalg)
tong --version-long  # extended version with git hash and build timestamp
tong --list-builtins # list core built-in functions (print, len, sum, map, filter, reduce, import)
```

Multi-line input: start a block with `{` (e.g. a function definition) and the prompt switches to `....` until braces balance.

Bare expressions echo their value automatically; use `print()` for formatted multi-value output.

## Built-in Functions (MVP)

- `print(...)` - Print values with formatted output
- `len(array)` - Get array or string length
- `sum(array)` - Sum array elements
- `map(array, funcName)` - Map function over array using a named function
- `filter(array, funcName)` - Keep elements for which the named function returns true
- `reduce(array, funcName, initial)` - Fold array with a named function taking (acc, item)

## Linear Algebra (Tensor) Module

Import the `linalg` module for basic multidimensional tensor support (MVP prototype):

```tong
let l = import("linalg")
let a = l.ones([2,2])
let b = l.ones([2,2])
let c = l.add(a,b)
print(l.shape(c))      // [2, 2]
print(l.get(c,[0,0]))  // 2.0
```

Provided functions:

| Function | Description |
|----------|-------------|
| `l.zeros(shape)` | Create tensor of zeros |
| `l.ones(shape)` | Create tensor of ones |
| `l.tensor(data, shape)` | Create tensor from flat numeric data and explicit shape |
| `l.shape(t)` | Returns shape array (e.g. `[2,3]`) |
| `l.rank(t)` | Returns rank (number of dimensions) |
| `l.get(t, idx)` | Get element at index list (e.g. `[i,j]`) |
| `l.set(t, idx, v)` | Returns a new tensor with element updated (immutable style) |
| `l.add(a,b)` | Elementwise addition (same shape) |
| `l.sub(a,b)` | Elementwise subtraction |
| `l.mul(a,b)` | Elementwise multiplication |
| `l.dot(a,b)` | Dot product of 1-D tensors (vectors) |
| `l.matmul(a,b)` | Matrix multiply rank‑2 tensors (m×k)·(k×n) -> (m×n) |
| `l.transpose(a)` | Transpose rank‑2 tensor |

Notes / Constraints (current MVP):

* All numeric values are stored as `f64` internally (ints are promoted on construction).
* No broadcasting yet — shapes must match for elementwise ops.
* `set` returns a new tensor (persistent style); no in‑place mutation.
* Only rank‑2 transpose is implemented now.
* Error messages are intentionally simple; richer diagnostics planned.

Example (see `examples/tensor.tong`):

```tong
let l = import("linalg")
let d = l.tensor([1,2,3,4],[2,2])
let e = l.transpose(d)
let f = l.matmul(d,e)
print(l.shape(f))            // [2, 2]
print(l.get(f,[0,0]))        // 5.0  (1*1 + 2*2)
print(l.get(f,[0,1]))        // 11.0 (1*3 + 2*4)
```

Planned next steps: broadcasting, slicing, reshaping, and sparse representations.

## Architecture

The Rust MVP consists of:

- `rust/tong/src/lexer.rs` - Tokenizes source code
- `rust/tong/src/parser.rs` - Builds an AST
- `rust/tong/src/runtime.rs` - Executes TONG programs
- `rust/tong/src/main.rs` - CLI entry point

## Language Design Principles

1. **Zero-cost abstractions** - High-level features compile to optimal code
2. **Memory safety** - Rust-inspired ownership without GC overhead
3. **Automatic parallelization** - Compiler parallelizes safe operations
4. **Heterogeneous computing** - Native support for diverse hardware
5. **Hot compilation** - REPL with JIT for interactive development

## Examples

The `examples/` directory contains demonstration programs:

- `hello.tong` - Basic syntax and output
- `math.tong` - Mathematical operations and functions
- `arrays.tong` - Array processing and built-ins
- `parallel.tong` - Parallel computing examples
- `advanced.tong` - Advanced language features
Additional examples and Rosetta tasks will be added as the Rust implementation evolves.

Module examples live under `examples/modules/`:
- `linalg` – tensors (creation, elementwise ops, dot, matmul, transpose, immutability, chaining)
- `sdl` – SDL3 Pong demo (graphics/input; requires `--features sdl3`)
See `examples/modules/README.md` for the index.

## Performance

TONG automatically optimizes code for performance:

- **Parallel sum**: Automatically uses multiple threads for large arrays
- **Parallel map**: Distributes work across available CPU cores
- **Memory efficiency**: Zero-copy operations where possible
- **Type optimization**: Specialized code paths for different types

## Development Status

🟢 **Completed (MVP)**
- Core language: variables, arithmetic, arrays
- Functions (definitions, calls, returns)
- If/else, comparisons, equality
- Built-ins: print, len, sum, map

🟡 **In Progress**
- filter, reduce built-ins
- Error spans and better diagnostics
- REPL
- WebAssembly backend

🔴 **Planned**
- LLVM backend for native compilation
- FPGA HDL generation
- Advanced type system
- Package management
- IDE integration

## Community & Contributing

Author: William Gacquer  
Company: AMILTO

We’re building TONG in the open — jump in!

Ways to contribute:
- Try the examples and report issues or ideas
- Tackle “good first issues” in the tracker
- Improve docs and examples (especially Rosetta tasks!)
- Add tests or small runtime/library utilities

Typical workflow:
1. Fork the repo and create a feature branch
2. Make focused, incremental changes (small PRs are best)
3. Add/adjust examples or tests as needed
4. Open a pull request and tell us what you improved

No contribution is too small — even typo fixes are appreciated. If you’re unsure where to start, open a GitHub Discussion or Issue and say hello.

## Example Output Regression Harness

All example programs under `examples/` are treated as golden tests. Their expected outputs (including warning lines) live in `examples/expected/` with the same relative path and a `.out` extension. The harness ensures language/runtime changes don’t silently alter behavior.

Generate / refresh every snapshot (non‑SDL examples):

```bash
bash scripts/gen_expected.sh
```

Run full regression check (fails on any diff):

```bash
bash scripts/check_examples.sh
```

Focused mode (only run specific examples):

```bash
FILES=hello.tong bash scripts/check_examples.sh
FILES="hello.tong math.tong" bash scripts/check_examples.sh
FILES=hello.tong,math.tong bash scripts/check_examples.sh   # commas also work
```

Auto‑update only failing/missing snapshots (use with review of git diff):

```bash
UPDATE=1 bash scripts/check_examples.sh
```

Combine focused & update:

```bash
FILES=features/pattern_clause_redundant.tong UPDATE=1 bash scripts/check_examples.sh
```

Run via Cargo tests (integration wrapper executes the harness):

```bash
cargo test --manifest-path rust/tong/Cargo.toml -- --nocapture
```

Typical workflow for intentional output changes:
1. Modify runtime / parser / examples.
2. Run the checker: `bash scripts/check_examples.sh` (see failing diffs).
3. Validate changes are desired.
4. Refresh just the changed snapshots: `UPDATE=1 bash scripts/check_examples.sh` (or regenerate all with `gen_expected.sh`).
5. Inspect and commit updated `.out` files with your code changes.

Notes:
- SDL examples are skipped (interactive / feature‑gated).
- Warning lines beginning with `[TONG][warn]` are asserted; wording changes require snapshot updates.
- CI runs the harness on every push & PR; mismatches fail the build.

Future enhancements (planned):
- Optional output normalization flags (timestamps, paths, etc.).
- DRY_RUN mode to preview updates without writing files.
- Parallel execution for faster runs on large example sets.
- Glob pattern filtering (e.g. `FILES='rosetta/*'`).


## Contributing

TONG is designed to be the ultimate programming language. Contributions are welcome!

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests and examples
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

---

TONG — The Ultimate Programming Language for Heterogeneous Computing
