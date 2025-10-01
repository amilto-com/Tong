# Module Examples Index

This directory groups example code organized by module. Each subfolder focuses on a specific built‑in or experimental module.

## Available Modules

### linalg
Experimental tensor support.

Run examples (from repo root with globally installed `tong`):
```bash
tong examples/modules/linalg/01_create.tong
```
Or from the Rust crate directory:
```bash
cd rust/tong
cargo run -- ../../examples/modules/linalg/01_create.tong
```
See `examples/modules/linalg/README.md` for the full list (creation, elementwise ops, dot, matmul, transpose, immutability, chained ops, error cases).

### sdl
SDL3 rendering / input demonstration via feature‑gated backend (`--features sdl3`).

Run Pong example (debug build):
```bash
cd rust/tong
cargo run --features sdl3 -- ../../examples/modules/sdl/pong.tong
```
If built without `--features sdl3` the runtime provides a headless shim (no window) and auto‑exits after simulated frames.

See `examples/modules/sdl/` for details.

## Adding a New Module
1. Create a subdirectory under `examples/modules/<name>`.
2. Provide a `README.md` describing the module’s purpose and how to run examples.
3. Add one or more `.tong` example files with ascending numeric prefixes for ordering.
4. Update this index file with a short section and link.

## Conventions
- File naming: `NN_description.tong` for ordered walkthroughs.
- Keep examples minimal and focused; prefer multiple small files over one large file.
- Include at least one file that demonstrates typical errors (commented out) for educational purposes.

---
Happy hacking in TONG’s modular ecosystem!
