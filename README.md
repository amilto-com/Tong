# TONG Programming Language

[![CI](https://github.com/amilto-com/Tong/actions/workflows/ci.yml/badge.svg)](https://github.com/amilto-com/Tong/actions/workflows/ci.yml)

<p align="center">
    <img src="images/TONG_a35c8343-4727-4443-bdaf-7ab3e9d3661d.jpg" alt="TONG Logo" width="320" />
</p>

The ultimate programming language designed for high‑performance parallel and distributed computing across heterogeneous environments (CPU/GPU/NPU/FPGA).

Authored by William Gacquer — AMILTO

Contributions welcome! See “Contributing” below.

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
```

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

## REPL

The Rust MVP currently runs .tong files. An interactive REPL is on the roadmap.

## Built-in Functions (MVP)

- `print(...)` - Print values with formatted output
- `len(array)` - Get array or string length
- `sum(array)` - Sum array elements
- `map(array, funcName)` - Map function over array using a named function
- `filter(array, funcName)` - Keep elements for which the named function returns true
- `reduce(array, funcName, initial)` - Fold array with a named function taking (acc, item)

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
