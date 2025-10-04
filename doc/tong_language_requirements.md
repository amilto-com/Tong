# Tong Language Implementation Requirements (Mandatory)

This document defines the non‑negotiable rules of Tong’s core semantics and surface syntax. All implementations, features, refactors, and examples MUST respect these requirements. When in doubt, prefer the stricter interpretation and emit a clear error.

## 1) Bindings, Assignment, and Scope

- let is immutable. Reassignment to a let binding MUST be rejected at runtime (or compile time) with a clear error.
- var is mutable. Only var bindings may be assigned to with name = expr and via array element update sugar name[idx] = expr.
- Function parameters and pattern bindings are let by default (immutable). To mutate a parameter, copy it into a local var (e.g., var n = n).
- Destructuring let (let (a,b) = arr) declares immutable names in the current scope.
- Shadowing: An inner scope MAY shadow outer names; assignment always targets the nearest existing binding by name.
- Blocks create a new lexical scope. A block’s last bare expression, if any, is the block’s value; otherwise it yields an empty array [].

## 2) Statements vs Expressions

- Assignment is statement‑level only. There is no assignment expression form.
- Return may appear in function bodies; otherwise, a function returns the value of its last bare expression implicitly.
- Array element update sugar name[idx] = v MUST produce a new array value (immutable update); negative indices MUST error; out‑of‑bounds MUST error.

## 3) Types and Values (Dynamic)

- Tong is dynamically typed. Supported core values include: Int (i64), Float (f64), Bool, Str, Array, Object (for module handles), Lambda, FuncRef, Constructor, Partial.
- Numeric division:
  - Int / Int yields Float. Modulo (%) is defined for Int only.
- Float equality MUST use an epsilon comparison for == and !=; do not rely on bit‑exact equality.

## 4) Operators and Precedence (highest → lowest)

1. Postfix: indexing, call, property
2. Unary: !, -, +
3. Multiplicative: *, /, %
4. Additive: +, -
5. Comparison: <, <=, >, >=
6. Equality: ==, !=
7. Logical AND: && (short‑circuit)
8. Logical OR: || (short‑circuit)
9. Bitwise: |, ^, & (non‑short‑circuit on Bool; Int bitwise)
10. Shifts: <<, >> (Int only; >> is logical right shift on the Int’s bit pattern)

Rules:
- All binary operators are left‑associative.
- Single‑amp (&) and single‑pipe (|) are bitwise; when applied to Bool, they are non‑short‑circuit logical combinators.
- Double‑amp && and double‑pipe || are logical with short‑circuit semantics.
- Parentheses may always override default grouping.

## 5) Functions, Lambdas, and Calls

- Named functions (fn) and lambdas are first‑class. Currying/partials are supported: calling with fewer args yields a Partial; equal arity fully applies.
- Lambda forms
  - Pipe single‑param: |x| x+1
  - Backslash: \x y -> x + y
  - fn with block: fn a b { ...; last_expr }
- Captured environments are lexically scoped and captured by value at lambda creation time.
- Arity MUST be enforced at call sites; too many/few args MUST be rejected (or return Partial if fewer).

## 6) Pattern Matching and Constructors

- data declarations register constructor names and arities globally.
- Constructor calls MUST be parenthesized when arity > 0: Just(42).
- Pattern matching supports wildcards (_), Int, Bool, tuple‑like array patterns, and constructors with nested subpatterns.
- Non‑exhaustive matches MUST error at runtime; implementations SHOULD emit warnings for redundancy and likely non‑exhaustiveness (heuristic is acceptable).
- Pattern bindings are immutable (let); they introduce new names in the match arm’s scope.

## 7) Guarded and Pattern‑Clause Functions

- Guarded multi‑clause resolution is top‑to‑bottom; the first passing guard executes.
- It is RECOMMENDED to include an unconditional final guard (if true) to avoid “no guard matched” runtime errors for total functions.
- Pattern‑clause (def) functions follow the same first‑match semantics. Arity across clauses MUST be consistent.

## 8) Arrays and Indexing

- Indexing evaluates target then index (left‑to‑right). The target MUST be an array; index MUST be Int ≥ 0 and < length; otherwise error.
- name[idx] = value is desugared to produce a new array value. Assigning it back to the same name requires that name be var.

## 9) Modules and Built‑ins

- import("module") loads a built‑in module and returns an Object exposing properties/methods.
- Built‑in modules: args, linalg, sdl (shim unless compiled with feature sdl3).
- args module exposes properties script, args, all and methods len/get/has/value; all are pure with no side‑effects.
- Built‑in functions include: now_ms, sleep_ms, range, repeat, sqrt, sin, cos, exp, log, abs, len, sum, map, filter, reduce, getenv.
- Module APIs MUST be stable and side‑effect behavior documented (e.g., sdl shim is deterministic and headless without the sdl3 feature).

## 10) Errors and Diagnostics

- Violations of mandatory rules MUST produce clear, user‑facing errors (e.g., “Cannot assign to immutable binding 'x' (use 'var' for mutable)”).
- Index and arity errors MUST be explicit. Avoid silent coercions.
- Implementations SHOULD include warnings for likely redundant/non‑exhaustive matches and unreachable guarded clauses.

## 11) Lexing, Syntax, and Tokens

- Comments: // to end of line.
- No statement terminator is required; semicolons are not part of the language.
- Keywords (reserved): let, var, fn, def, data, match, if, else, while, return, parallel, import, true, false, in.
- The '|' character is overloaded: in lambda pipe form |x| ... and as bitwise OR. The parser MUST disambiguate by context: expression layer for bitwise, atom layer for lambda literal.

## 12) CLI and Execution Model

- CLI MUST support trailing script arguments, including hyphenated values, without being parsed as Tong flags.
- The execution environment provides args via the args module; implementing execute_with_cli(script, args) is REQUIRED.
- Parallel blocks may be implemented sequentially today but MUST preserve left‑to‑right evaluation order of contained statements.

## 13) Determinism and Side‑Effects

- Except for time/sleep/getenv, print, and module I/O (e.g., sdl), evaluation SHOULD be pure and deterministic.
- Short‑circuit operator semantics MUST NOT evaluate the right operand when not required (for && and || only).

## 14) Compatibility and Migration (let/var)

- Legacy code relying on implicit mutability MUST be updated to var for reassignments and element updates.
- Examples, docs, and tests MUST use let for immutable values and var for state that changes.

---

Non‑compliance with these rules is considered a defect. Proposed extensions MUST state whether they refine, extend, or deliberately relax any requirement above, and include a migration note.
