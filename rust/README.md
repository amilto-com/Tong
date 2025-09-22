# TONG (Rust MVP)

A minimal Rust implementation to run the simplest TONG examples while we migrate off Python.

What works now:
- Parse and run a subset: `fn main() { print(...); let ... = ...; print(...); }` and a trailing `main()` call
- String, float, int literals; identifiers in prints are looked up in a simple environment

How to run:

```powershell
cd rust/tong
cargo run -- ../../examples/hello.tong
```

Next steps:
- Extend lexer/parser to cover more syntax from `tong_spec.md`
- Implement variables, expressions, and more statements
- Add unit tests
- Later: integrate with the original Python runtime behaviors or replace them feature-by-feature
