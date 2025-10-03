# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project adheres (aspirationally) to **Semantic Versioning**.

## [Unreleased]
### Added
- (placeholder) Future changes go here.

## [0.1.0] - 2025-10-03
### Added
- Core language implementation (lexer, parser, runtime, CLI) and initial examples (`a4a954f`, `675de17`).
- PowerShell and shell setup scripts for cross‑platform installation (`9408342`, `7c29f7f`).
- Rosetta examples collection and classic algorithm demonstrations (`57f512c`).
- Linear algebra (tensor) module with creation, elementwise ops, dot, matmul, transpose (`0d64f3b`, `e6ba797`, `f8430e1`).
- REPL with interactive commands (`9180358`).
- Experimental functional feature set inspired by Haskell: data declarations, pattern matching, lambdas, partial application (`2a3c24d`, `d9fcfc9`).
- Multi-generator list comprehensions (`e68c1a4`).
- Constructor calls and nested constructor patterns in expressions (`3baccf9`, `1223b52`, `3e0d4f1`).
- Match arm redundancy, reachability and basic exhaustiveness heuristics (`05a1ef8`, `8202a1b`, `4d686d9`).
- Pattern function clause redundancy & reachability warnings (`fcc5f86`).
- Semantic constructor detection via `known_ctors` map, eliminating heuristic ambiguity (`f835d86`).
- Example regression harness and snapshot generation (`2013208`, `15a504b`, `58017a7`).
- Focused FILES mode, UPDATE=1 snapshot refresh, cargo test integration (`66c041f`).
- CI workflow (fmt, clippy, build, harness) and status badge (`e53043c`).
- Extended README sections (architecture, modules, harness docs) (`8211450`, `11424f2`, `512bd7a`).

### Changed
- Parser refactored for semantic constructor classification and improved pattern analysis (`f835d86`, `4d686d9`).
- Code structure and readability improvements (`55288b8`).
- README expanded for new features & harness usage (`512bd7a`).

### Fixed
- Misclassification of single-letter uppercase identifiers as constructors (semantic detection) (`f835d86`).
- Object argument call handling & SDL backend stability (`df651b3`).
- Early arithmetic increment issue in harness under `set -euo pipefail` (use pre-increment) (`2013208`).
- Various lint/style issues (clippy & rustfmt passes) (`3c7919f`, `3910d2b`).

### Removed
- Legacy Python implementation and scripts; project is now Rust-only (`d5bf6fb`).

### Security
- Treat all Rust warnings as errors in CI to prevent latent undefined behavior patterns (`e53043c`).

### Notes
- Version number reflects crate `0.1.0`; future releases will increment per semantic versioning once public API contracts are defined.
- SDL examples intentionally excluded from harness (interactive / feature-gated).

[Unreleased]: https://github.com/amilto-com/Tong/compare/0.1.0...HEAD
[0.1.0]: https://github.com/amilto-com/Tong/releases/tag/0.1.0
