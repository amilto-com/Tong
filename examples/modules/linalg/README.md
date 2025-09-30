# Linalg Module Examples

This directory showcases usage of the experimental `linalg` tensor module.

Each file is self‑contained; run one with:

```bash
cd rust/tong
cargo run -- ../../examples/modules/linalg/01_create.tong
```
(or using a globally installed `tong` binary from repo root: `tong examples/modules/linalg/01_create.tong`)

## Files

1. `01_create.tong` – zeros/ones, shape, rank, element access
2. `02_tensor_constructor.tong` – explicit `tensor(data, shape)` constructor
3. `03_elementwise.tong` – elementwise add/sub/mul
4. `04_dot.tong` – vector dot product
5. `05_matmul_basic.tong` – basic 2×2 matrix multiplication
6. `06_transpose.tong` – transpose of a 2×3 matrix
7. `07_set_immutability.tong` – `set` returns a new tensor (immutability)
8. `08_chain_ops.tong` – chaining elementwise ops with transpose & matmul
9. `09_errors.tong` – commented examples of typical errors

## Notes
- All numeric data is stored internally as `f64`.
- Shapes must match for elementwise ops (no broadcasting yet).
- `set` does not mutate in place; it returns a new tensor.
- Transpose only implemented for rank‑2 tensors.

Planned improvements: reshape, broadcast, slicing.
