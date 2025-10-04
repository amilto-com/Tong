# Tong Gradual Typing Guide

This document explains how to use optional type annotations in Tong.

- You can annotate let/var bindings:
  - let x: Int = 42
  - var name: Str = "mozza"
  - let v: Array = [1,2,3]
  - Use Any to opt-out: let x: Any = foo()

- You can annotate simple function parameters and return types:
  - fn add(a: Int, b: Int) -> Int { return a + b }
  - fn head(xs: Array) -> Any { return xs[0] }

Rules and behavior:
- Annotations are enforced at runtime for now:
  - Binding annotations (LetAnn/VarAnn) check the right-hand side value.
  - Function parameter annotations check arguments at call time.
  - Function return annotations check the returned value when the function completes.
- Guarded functions support annotations on simple parameters and an optional return type.
- Pattern functions support an optional return type annotation (per-parameter annotations for patterns are not supported yet).
- The type keywords are: Int, Float, Bool, Str, Array, Any.
- Any disables checking for that annotation.

Static lint
- A lightweight lint pass runs before execution and emits warnings for obvious mismatches when the right-hand side is a literal or when a function has an explicit return of a literal.

Examples:
- See examples/typed/ for small annotated snippets.
