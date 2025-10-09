# Tong WASM Examples

This directory contains WebAssembly examples for the Tong programming language.

## Quick Start

1. Compile a Tong program to WASM:
   ```bash
   cd /data/Tong
   ./rust/tong/target/release/tong --wasm examples/wasm_test.tong
   ```

2. Run in browser:
   ```bash
   cd examples
   python3 -m http.server 8000
   ```
   
3. Open http://localhost:8000/wasm_runner.html

## Files

- `wasm_test.tong` - Simple Tong program demonstrating functions and arithmetic
- `wasm_test.wasm` - Compiled WebAssembly binary (generated)
- `wasm_test.wat` - WebAssembly text format (generated)
- `wasm_runner.html` - Browser-based WASM execution environment

## Compile Options

```bash
# Compile to WASM binary
tong --wasm program.tong

# Compile to WAT (text format)
tong --wat program.tong

# Specify output file
tong --wasm -o custom.wasm program.tong
tong --wat -o custom.wat program.tong
```

## Viewing WAT Output

The WAT (WebAssembly Text) format is human-readable:

```bash
tong --wat examples/wasm_test.tong
cat examples/wasm_test.wat
```

## Creating Your Own WASM Programs

### Supported Features

✅ Variables (let, var)
✅ Arithmetic (+, -, *, /, %)
✅ Comparisons (==, !=, <, <=, >, >=)
✅ Logical operations (&&, ||, !)
✅ Functions with parameters
✅ Function calls
✅ Print statements

### Example Program

```tong
// my_program.tong
let a = 10
let b = 20

fn square(x) {
    x * x
}

print(a + b)
print(square(5))
```

Compile and run:
```bash
tong --wasm my_program.tong
# Then load in browser using wasm_runner.html
```

## Browser Compatibility

- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

## Troubleshooting

**Module not loading?**
- Make sure you're using an HTTP server, not file:// protocol
- Check browser console for errors
- Verify WASM file was generated successfully

**Print not working?**
- Ensure import functions are defined in importObject
- Check that function signatures match

## Learn More

See [doc/wasm_backend.md](../doc/wasm_backend.md) for complete documentation.
