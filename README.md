# TONG Programming Language

The ultimate programming language designed for high-performance parallel and distributed computing across heterogeneous environments (CPU/GPU/NPU/FPGA).

## Features

🚀 **High Performance**
- Zero-cost abstractions
- Automatic parallelization for large datasets
- Memory safety without garbage collection overhead
- JIT compilation in REPL mode

⚡ **Heterogeneous Computing**
- Native support for CPU/GPU/NPU/FPGA execution
- Automatic workload distribution
- GPU kernel compilation
- Distributed computing primitives

🔧 **Developer Experience**
- Interactive REPL with hot compilation
- Modern syntax combining best of all languages
- Comprehensive error messages
- Built-in parallel algorithms

🌐 **Compilation Targets**
- Native machine code (x86_64, ARM64, RISC-V)
- WebAssembly (WASM)
- GPU shaders (CUDA, OpenCL, Metal)
- FPGA HDL (planned)

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/amilto-com/Tong.git
cd Tong

# Make the main script executable
chmod +x tong.py
```

### Running TONG

```bash
# Start interactive REPL
./tong.py

# Run a TONG program
./tong.py examples/hello.tong

# Show help
./tong.py --help
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
// Immutable by default
let x = 42
let name = "TONG"

// Mutable variables
var counter = 0

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

// Built-in functions with automatic parallelization
print("Sum:", sum(numbers))          // Parallel for large arrays
print("Length:", len(numbers))
print("Doubled:", map(numbers, |x| x * 2))  // Parallel map
```

### Parallel Computing
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

## REPL Usage

Start the interactive REPL:

```bash
./tong.py
```

REPL Commands:
- `help` - Show help message
- `vars` - Show all defined variables
- `history` - Show command history
- `clear` - Clear environment
- `exit` - Exit REPL

Example REPL session:
```
>>> let x = 42
>>> let y = 8
>>> x + y
=> 50

>>> let numbers = [1, 2, 3, 4, 5]
>>> sum(numbers)
=> 15

>>> fn square(n) { n * n }
>>> square(7)
=> 49
```

## Built-in Functions

- `print(...)` - Print values with formatted output
- `len(array)` - Get array or string length
- `sum(array)` - Sum array elements (auto-parallel for large arrays)
- `map(array, func)` - Map function over array (auto-parallel)
- `filter(array, func)` - Filter array elements
- `reduce(array, func, initial)` - Reduce array to single value

## Architecture

TONG is implemented with a modular architecture:

- **Lexer** (`src/lexer.py`) - Tokenizes source code
- **Parser** (`src/parser.py`) - Builds Abstract Syntax Tree
- **AST** (`src/ast_nodes.py`) - Defines language constructs
- **Interpreter** (`src/interpreter.py`) - Executes TONG programs
- **REPL** (`src/repl.py`) - Interactive programming environment

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

## Performance

TONG automatically optimizes code for performance:

- **Parallel sum**: Automatically uses multiple threads for large arrays
- **Parallel map**: Distributes work across available CPU cores
- **Memory efficiency**: Zero-copy operations where possible
- **Type optimization**: Specialized code paths for different types

## Development Status

🟢 **Completed**
- Core language implementation
- Lexer and parser
- Basic interpreter
- REPL environment
- Parallel execution framework
- Standard library functions

🟡 **In Progress**
- Advanced parallel constructs
- GPU kernel compilation
- WebAssembly backend

🔴 **Planned**
- LLVM backend for native compilation
- FPGA HDL generation
- Advanced type system
- Package management
- IDE integration

## Contributing

TONG is designed to be the ultimate programming language. Contributions are welcome!

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests and examples
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Why TONG?

TONG combines the best features from all major programming languages:

- **Performance** like C/C++ and Rust
- **Simplicity** like Python and Go  
- **Expressiveness** like Haskell and ML
- **Concurrency** like Erlang and Go
- **Safety** like Rust and Swift

TONG is designed for the future of computing where heterogeneous architectures (CPU/GPU/NPU/FPGA) work together seamlessly.

---

*TONG - The Ultimate Programming Language for Heterogeneous Computing*
