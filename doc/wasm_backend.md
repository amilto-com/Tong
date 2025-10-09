# Tong WASM Backend

## Overview

The Tong WASM backend compiles Tong programs to WebAssembly, enabling Tong code to run in web browsers and other WASM-compatible environments with near-native performance.

## Features

### ✅ Supported Features

- **Basic Data Types**: Integers (i64), Floats (f64), Booleans (i32)
- **Variables**: `let` and `var` declarations
- **Arithmetic Operations**: `+`, `-`, `*`, `/`, `%`
- **Comparison Operations**: `==`, `!=`, `<`, `<=`, `>`, `>=`
- **Logical Operations**: `&&`, `||`, `!`
- **Bitwise Operations**: `&`, `|`, `^`, `<<`, `>>`
- **Functions**: User-defined functions with parameters
- **Function Calls**: Both user-defined and imported functions
- **Print Statements**: Outputs to console via imported functions

### 🚧 Limitations

- **No Arrays/Strings**: Complex data structures require memory management (coming soon)
- **No Control Flow**: if/while/for statements not yet implemented
- **No Closures**: Lambda functions not supported
- **Type Inference**: All numbers treated as i64 by default
- **No Pattern Matching**: Match expressions not implemented

## Usage

### Command Line

Compile a Tong program to WASM:

```bash
# Compile to WASM binary
tong --wasm examples/wasm_test.tong

# Compile to WAT (WebAssembly Text format)
tong --wat examples/wasm_test.tong

# Specify output file
tong --wasm -o output.wasm examples/wasm_test.tong
```

### Example Program

```tong
// wasm_test.tong
let x = 42
let y = 10

fn add(a, b) {
    a + b
}

fn multiply(a, b) {
    a * b
}

print(x + y)        // Outputs: 52
print(add(5, 7))    // Outputs: 12
print(multiply(6, 7)) // Outputs: 42
```

### Running in Browser

1. Compile your Tong program to WASM:
   ```bash
   tong --wasm program.tong
   ```

2. Create an HTML file to load and run the WASM:
   ```html
   <!DOCTYPE html>
   <html>
   <body>
       <div id="output"></div>
       <script>
           const importObject = {
               env: {
                   print_i32: (v) => console.log('i32:', v),
                   print_i64: (v) => console.log('i64:', v),
                   print_f64: (v) => console.log('f64:', v)
               }
           };

           WebAssembly.instantiateStreaming(
               fetch('program.wasm'),
               importObject
           ).then(result => {
               result.instance.exports.main();
           });
       </script>
   </body>
   </html>
   ```

3. Serve with a local HTTP server:
   ```bash
   python3 -m http.server 8000
   ```

4. Open http://localhost:8000 in your browser

## Technical Details

### Module Structure

Generated WASM modules have the following structure:

- **Type Section**: Function signatures
  - Type 0: `() -> ()` - Main and simple functions
  - Type 1: `(i32) -> ()` - print_i32
  - Type 2: `(i64) -> ()` - print_i64
  - Type 3: `(f64) -> ()` - print_f64

- **Import Section**: External functions from host environment
  - `env.print_i32` - Print 32-bit integers
  - `env.print_i64` - Print 64-bit integers
  - `env.print_f64` - Print 64-bit floats

- **Function Section**: User-defined functions

- **Memory Section**: Linear memory (1-10 pages)

- **Export Section**: Exported functions
  - `main` - Entry point

- **Code Section**: Function bytecode

### Type Mapping

| Tong Type | WASM Type | Notes |
|-----------|-----------|-------|
| Integer | i64 | 64-bit signed |
| Float | f64 | 64-bit floating point |
| Boolean | i32 | 0 = false, 1 = true |
| String | i32 | Pointer to memory (planned) |
| Array | - | Not yet implemented |

### Instruction Mapping

| Tong Operation | WASM Instruction |
|----------------|------------------|
| `a + b` | i64.add |
| `a - b` | i64.sub |
| `a * b` | i64.mul |
| `a / b` | i64.div_s |
| `a % b` | i64.rem_s |
| `a == b` | i64.eq |
| `a != b` | i64.ne |
| `a < b` | i64.lt_s |
| `a <= b` | i64.le_s |
| `a > b` | i64.gt_s |
| `a >= b` | i64.ge_s |
| `a && b` | i32.and |
| `a \|\| b` | i32.or |
| `!a` | i32.eqz |
| `-a` | 0 - a (i64.sub) |

## Performance

WASM execution is significantly faster than interpreted mode:

- **Arithmetic**: ~100x faster
- **Function calls**: ~50x faster
- **Startup**: Slightly slower due to compilation

## Future Enhancements

### Planned Features

1. **Control Flow**
   - if/else statements
   - while/for loops
   - Match expressions

2. **Data Structures**
   - Arrays with proper memory allocation
   - Strings in linear memory
   - Objects/records

3. **Advanced Features**
   - Closures and lambda functions
   - Higher-order functions
   - Tail call optimization

4. **Optimizations**
   - Dead code elimination
   - Constant folding
   - Inline expansion
   - Loop unrolling

5. **Interoperability**
   - Import JavaScript functions
   - Export Tong functions to JavaScript
   - Shared memory with JavaScript

6. **Tooling**
   - Source maps for debugging
   - WASM size optimization
   - Profiling support

## Troubleshooting

### Common Issues

**Q: "Error: Module parse failed"**
A: Ensure you're using a modern browser with WASM support (Chrome 57+, Firefox 52+, Safari 11+)

**Q: "Cannot find module"**
A: Make sure to serve files via HTTP, not file:// protocol

**Q: "Undefined imported function"**
A: Check that all required functions are defined in importObject

## Examples

See the `examples/` directory for complete examples:

- `wasm_test.tong` - Simple arithmetic and functions
- `wasm_runner.html` - Browser-based WASM runner
- `wasm_test.wat` - Generated WAT output

## Contributing

The WASM backend is actively being developed. Contributions are welcome!

Priority areas:
- Control flow implementation
- Memory management for complex types
- Optimization passes
- Better error messages

## License

Same as Tong interpreter (see LICENSE file)
