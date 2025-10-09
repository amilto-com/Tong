use anyhow::{bail, Result};

use crate::value::Value;

use crate::parser::{Expr, Pattern, Stmt, TypeAnn, Program};
use crate::env::Env;
use crate::value::format_value;

pub fn builtin_modules() -> Vec<&'static str> {
    vec!["sdl", "linalg", "args"]
}

// Core built-in functions (non-module) recognized directly by the evaluator.
// Keep in sync with match arms in Expr::Call in eval_expr.
pub fn builtin_functions() -> Vec<&'static str> {
    // print is a statement in the AST but users expect it; include for discoverability.
    let mut v = vec![
        "print", "len", "sum", "filter", "reduce", "map", "import",
        // New general-purpose helpers useful for benchmarks and C-like tasks
        "now_ms", "sleep_ms", "range", "repeat", "sqrt", "sin", "cos", "atan", "exp", "log", "abs",
        "getenv",
    ];
    v.sort();
    v
}

#[allow(dead_code)]
pub fn execute(program: Program, debug: bool) -> Result<()> {
    let mut env = Env::new();
    env.debug = debug;
    if env.debug {
        eprintln!(
            "[TONG][dbg] start: {} top-level statements",
            program.stmts.len()
        );
    }

    // First collect function definitions
    for (i, stmt) in program.stmts.iter().enumerate() {
        if env.debug {
            eprintln!("[TONG][dbg] top-level stmt #{}: {}", i, stmt.kind_name());
        }
        match stmt {
            Stmt::FnDef(name, params, body) => {
                env.funcs
                    .insert(name.clone(), (params.clone(), body.clone()));
            }
            Stmt::FnDefTyped(name, params, ret_ann, body) => {
                let p: Vec<String> = params.iter().map(|(n, _)| n.clone()).collect();
                env.funcs.insert(name.clone(), (p, body.clone()));
                env.fn_types.insert(
                    name.clone(),
                    (
                        params.iter().map(|(_, t)| t.clone()).collect(),
                        ret_ann.clone(),
                    ),
                );
            }
            Stmt::FnDefGuarded(name, params, guard, body) => {
                env.guarded_funcs.entry(name.clone()).or_default().push((
                    params.clone(),
                    guard.clone(),
                    body.clone(),
                ));
            }
            Stmt::FnDefGuardedTyped(name, params, ret_ann, guard, body) => {
                env.fn_types.insert(
                    name.clone(),
                    (
                        params.iter().map(|(_, t)| t.clone()).collect(),
                        ret_ann.clone(),
                    ),
                );
                env.guarded_funcs.entry(name.clone()).or_default().push((
                    params.iter().map(|(n, _)| n.clone()).collect(),
                    guard.clone(),
                    body.clone(),
                ));
            }
            Stmt::FnDefPattern(name, patterns, guard, ret_ann, body) => {
                env.pattern_funcs.entry(name.clone()).or_default().push((
                    patterns.clone(),
                    guard.clone(),
                    ret_ann.clone(),
                    body.clone(),
                ));
            }
            Stmt::FnMain(body) => {
                env.funcs
                    .insert("main".to_string(), (Vec::new(), body.clone()));
            }
            Stmt::DataDecl(_, ctors) => {
                for c in ctors {
                    env.data_ctors.insert(c.name.clone(), c.arity);
                }
            }
            _ => {}
        }
    }

    // Basic redundancy / ordering warnings for pattern function clauses (heuristic).
    // Warn if an all-wildcard clause is not last, or if a clause is strictly unreachable
    // because an earlier clause has an equivalent key with no guard.
    for (fname, clauses) in &env.pattern_funcs {
        // Find all-wildcard position (all params wildcard / identifiers treated as wildcard)
        let mut wildcard_pos: Option<usize> = None;
        for (idx, (pats, guard, _rann, _body)) in clauses.iter().enumerate() {
            if guard.is_none()
                && pats
                    .iter()
                    .all(|p| matches!(p, Pattern::Wildcard | Pattern::Ident(_)))
            {
                wildcard_pos = Some(idx);
                break;
            }
        }
        if let Some(wi) = wildcard_pos {
            if wi + 1 < clauses.len() && std::env::var("TONG_NO_MATCH_WARN").is_err() {
                for later in (wi + 1)..clauses.len() {
                    eprintln!(
                        "[TONG][warn] unreachable pattern function clause #{} for '{}' (preceded by all-wildcard clause #{})",
                        later, fname, wi
                    );
                }
            }
        }
        // Duplicate key detection: create a simplified key per clause (ignores guards when present)
        fn pat_key(p: &Pattern) -> String {
            match p {
                Pattern::Wildcard | Pattern::Ident(_) => "_".to_string(),
                Pattern::Int(i) => format!("i:{}", i),
                Pattern::Bool(b) => format!("b:{}", b),
                Pattern::Constructor { name, sub, .. } => {
                    if sub.is_empty() {
                        format!("C:{}", name)
                    } else {
                        let inner: Vec<String> = sub.iter().map(pat_key).collect();
                        format!("C:{}({})", name, inner.join(","))
                    }
                }
                Pattern::Tuple(subs) => {
                    let inner: Vec<String> = subs.iter().map(pat_key).collect();
                    format!("T({})", inner.join(","))
                }
            }
        }
        // Helper: structural subsumption for patterns
        fn pat_subsumes(a: &Pattern, b: &Pattern) -> bool {
            use Pattern::*;
            match (a, b) {
                (Wildcard, _) => true,
                (Ident(_), _) => true,
                (Int(i), Int(j)) => i == j,
                (Bool(x), Bool(y)) => x == y,
                (
                    Constructor {
                        name: n1, sub: s1, ..
                    },
                    Constructor {
                        name: n2, sub: s2, ..
                    },
                ) => {
                    n1 == n2
                        && s1.len() == s2.len()
                        && s1.iter().zip(s2.iter()).all(|(p, q)| pat_subsumes(p, q))
                }
                (Tuple(ts1), Tuple(ts2)) => {
                    ts1.len() == ts2.len()
                        && ts1.iter().zip(ts2.iter()).all(|(p, q)| pat_subsumes(p, q))
                }
                _ => false,
            }
        }
        fn clause_subsumes(prev: &[Pattern], next: &[Pattern]) -> bool {
            if prev.len() != next.len() {
                return false;
            }
            prev.iter()
                .zip(next.iter())
                .all(|(a, b)| pat_subsumes(a, b))
        }
        let mut seen: Vec<(String, usize, Vec<Pattern>)> = Vec::new();
        for (idx, (pats, guard, _rann, _)) in clauses.iter().enumerate() {
            if guard.is_some() {
                continue; // Guards may differentiate runtime reachability; skip for now
            }
            let key = pats.iter().map(pat_key).collect::<Vec<_>>().join("|");
            if let Some((_, prev_idx, _)) = seen.iter().find(|(k, _, _)| k == &key) {
                if std::env::var("TONG_NO_MATCH_WARN").is_err() {
                    eprintln!(
                        "[TONG][warn] redundant pattern function clause #{} for '{}' (covered by earlier clause #{})",
                        idx, fname, prev_idx
                    );
                }
                continue;
            }
            // Subsumption: if an earlier key is identical at each position or is a wildcard ('_'), treat this clause as unreachable
            // Also, constructor-specific: a prior constructor with same name and arity subsumes identical subpattern structures.
            let parts: Vec<String> = pats.iter().map(pat_key).collect();
            for (prev_key, prev_idx, prev_pats) in seen.iter() {
                let prev_parts: Vec<&str> = prev_key.split('|').collect();
                if prev_parts.len() == parts.len() {
                    let mut subsumes = true;
                    for (a, b) in prev_parts.iter().zip(parts.iter()) {
                        if *a == "_" {
                            continue;
                        }
                        if *a != b {
                            subsumes = false;
                            break;
                        }
                    }
                    if (subsumes || clause_subsumes(prev_pats, pats))
                        && std::env::var("TONG_NO_MATCH_WARN").is_err()
                    {
                        eprintln!(
                            "[TONG][warn] unreachable pattern function clause #{} for '{}' (subsumed by earlier clause #{})",
                            idx, fname, prev_idx
                        );
                        break;
                    }
                }
            }
            seen.push((key, idx, pats.clone()));
        }
    }

    // Lightweight type linting for annotated code
    for msg in lint_types(&program) {
        eprintln!("[TONG][lint] {msg}");
    }

    // Execute top-level statements (import/let/assign/print)
    for stmt in &program.stmts {
        match stmt {
            Stmt::Import(name, module) => {
                let v = env.import_module(&module)?;
                env.vars_mut().insert(name.clone(), v);
            }
            Stmt::Let(name, expr) => {
                let v = env.eval_expr(expr.clone())?;
                env.declare_let(name.clone(), v);
            }
            Stmt::Var(name, expr) => {
                let v = env.eval_expr(expr.clone())?;
                env.declare_var(name.clone(), v);
            }
            Stmt::Assign(name, expr) => {
                let v = env.eval_expr(expr.clone())?;
                env.assign_var(&name, v)?;
            }
            Stmt::ArrayAssign(name, idx_expr, val_expr) => {
                let idx_v = env.eval_expr(idx_expr.clone())?;
                let new_v = env.eval_expr(val_expr.clone())?;
                match idx_v {
                    Value::Int(i) => env.array_assign(&name, i, new_v)?,
                    _ => bail!("array element assignment expects array variable and int index"),
                }
            }
            Stmt::LetTuple(names, expr) => {
                let v = env.eval_expr(expr.clone())?;
                match v {
                    Value::Array(items) => {
                        if items.len() != names.len() {
                            bail!("tuple arity mismatch");
                        }
                        for (n, it) in names.iter().zip(items.into_iter()) {
                            env.declare_let(n.clone(), it);
                        }
                    }
                    _ => bail!("destructuring expects array value"),
                }
            }
            Stmt::Print(args) => {
                let parts: Result<Vec<String>> = args
                    .iter()
                    .cloned()
                    .map(|e| env.eval_expr(e).map(|v| format_value(&v)))
                    .collect();
                println!("{}", parts?.join(" "));
            }
            Stmt::Expr(e) => {
                let _ = env.eval_expr(e.clone())?;
            }
            Stmt::DataDecl(type_name, ctors) => {
                let mut names = Vec::new();
                for c in ctors {
                    env.data_ctors.insert(c.name.clone(), c.arity);
                    names.push(c.name.clone());
                }
                env.type_ctors.insert(type_name.clone(), names);
            }
            Stmt::FnDefGuarded(_, _, _, _) => { /* already collected */ }
            Stmt::FnDefGuardedTyped(_, _, _, _, _) => { /* already collected */ }
            // Allow control-flow at top-level
            Stmt::If(..) | Stmt::While(..) | Stmt::Parallel(..) | Stmt::Return(..) => {
                let _ = env.exec_stmt(&stmt)?;
            }
            _ => {}
        }
    }

    Ok(())
}

// Simple lint/static pass for annotated code: checks let/var annotations w/ literal RHS and typed fn returns with literal return
pub fn lint_types(program: &Program) -> Vec<String> {
    let mut out = Vec::new();
    fn infer_expr_type(e: &Expr) -> Option<TypeAnn> {
        match e {
            Expr::Int(_) => Some(TypeAnn::Int),
            Expr::Float(_) => Some(TypeAnn::Float),
            Expr::Bool(_) => Some(TypeAnn::Bool),
            Expr::Str(_) => Some(TypeAnn::Str),
            Expr::Array(_) => Some(TypeAnn::Array),
            _ => None,
        }
    }
    for s in &program.stmts {
        match s {
            Stmt::LetAnn(_, ann, rhs) | Stmt::VarAnn(_, ann, rhs) => {
                if let Some(it) = infer_expr_type(&rhs) {
                    if *ann != TypeAnn::Any && *ann != it {
                        out.push(format!(
                            "annotation mismatch: expected {:?}, got {:?}",
                            ann, it
                        ));
                    }
                }
            }
            Stmt::FnDefTyped(name, _params, Some(rann), body) => {
                for st in body {
                    if let Stmt::Return(e) = st {
                        if let Some(it) = infer_expr_type(e) {
                            if *rann != TypeAnn::Any && *rann != it {
                                out.push(format!(
                                    "function {} return annotation mismatch: expected {:?}, got {:?}",
                                    name, rann, it
                                ));
                            }
                        }
                    }
                }
            }
            Stmt::FnDefGuardedTyped(name, _params, Some(rann), _guard, body) => {
                for st in body {
                    if let Stmt::Return(e) = st {
                        if let Some(it) = infer_expr_type(e) {
                            if *rann != TypeAnn::Any && *rann != it {
                                out.push(format!(
                                    "function {} return annotation mismatch: expected {:?}, got {:?}",
                                    name, rann, it
                                ));
                            }
                        }
                    }
                }
            }
            Stmt::FnDefPattern(name, _pats, _guard, Some(rann), body) => {
                for st in body {
                    if let Stmt::Return(e) = st {
                        if let Some(it) = infer_expr_type(e) {
                            if *rann != TypeAnn::Any && *rann != it {
                                out.push(format!(
                                    "function {} return annotation mismatch: expected {:?}, got {:?}",
                                    name, rann, it
                                ));
                            }
                        }
                    }
                }
            }
            Stmt::FnDefTyped(_, _, None, _) => {}
            _ => {}
        }
    }
    // Pattern function unreachable clause lint (structural subsumption, ignoring guarded clauses)
    use std::collections::HashMap;
    type PatClauses = HashMap<String, Vec<(Vec<Pattern>, Option<Expr>)>>;
    let mut pat_funcs: PatClauses = HashMap::new();
    for s in &program.stmts {
        if let Stmt::FnDefPattern(name, pats, guard, _rann, _body) = s {
            pat_funcs
                .entry(name.clone())
                .or_default()
                .push((pats.clone(), guard.clone()));
        }
    }
    fn pat_subsumes(a: &Pattern, b: &Pattern) -> bool {
        match (a, b) {
            (Pattern::Wildcard, _) => true,
            (Pattern::Ident(_), _) => true,
            (Pattern::Int(i), Pattern::Int(j)) => i == j,
            (Pattern::Bool(x), Pattern::Bool(y)) => x == y,
            (
                Pattern::Constructor {
                    name: n1, sub: s1, ..
                },
                Pattern::Constructor {
                    name: n2, sub: s2, ..
                },
            ) => {
                n1 == n2
                    && s1.len() == s2.len()
                    && s1.iter().zip(s2.iter()).all(|(p, q)| pat_subsumes(p, q))
            }
            (Pattern::Tuple(ts1), Pattern::Tuple(ts2)) => {
                ts1.len() == ts2.len()
                    && ts1.iter().zip(ts2.iter()).all(|(p, q)| pat_subsumes(p, q))
            }
            _ => false,
        }
    }
    fn clause_subsumes(prev: &[Pattern], next: &[Pattern]) -> bool {
        prev.len() == next.len()
            && prev
                .iter()
                .zip(next.iter())
                .all(|(a, b)| pat_subsumes(a, b))
    }
    for (fname, clauses) in pat_funcs {
        let mut seen: Vec<(Vec<Pattern>, usize)> = Vec::new();
        for (idx, (pats, guard)) in clauses.iter().enumerate() {
            if guard.is_some() {
                continue;
            }
            for (prev_pats, prev_idx) in seen.iter() {
                if clause_subsumes(prev_pats, pats) {
                    out.push(format!(
                        "unreachable pattern function clause #{} for '{}' (subsumed by earlier clause #{})",
                        idx, fname, prev_idx
                    ));
                    break;
                }
            }
            seen.push((pats.clone(), idx));
        }
    }
    out
}

pub fn execute_with_cli(
    program: Program,
    debug: bool,
    script: Option<String>,
    args: Vec<String>,
) -> Result<()> {
    let mut env = Env::new();
    env.debug = debug;
    env.cli_script = script;
    env.cli_args = args;
    if env.debug {
        eprintln!(
            "[TONG][dbg] start: {} top-level statements (with CLI)",
            program.stmts.len()
        );
    }
    // Reuse main execute logic: collect definitions and run top-level
    // First collect function definitions
    for (i, stmt) in program.stmts.iter().enumerate() {
        if env.debug {
            eprintln!("[TONG][dbg] top-level stmt #{}: {}", i, stmt.kind_name());
        }
        match stmt {
            Stmt::FnDef(name, params, body) => {
                env.funcs
                    .insert(name.clone(), (params.clone(), body.clone()));
            }
            Stmt::FnDefTyped(name, params, ret_ann, body) => {
                let p: Vec<String> = params.iter().map(|(n, _)| n.clone()).collect();
                env.funcs.insert(name.clone(), (p, body.clone()));
                env.fn_types.insert(
                    name.clone(),
                    (
                        params.iter().map(|(_, t)| t.clone()).collect(),
                        ret_ann.clone(),
                    ),
                );
            }
            Stmt::FnDefGuarded(name, params, guard, body) => {
                env.guarded_funcs.entry(name.clone()).or_default().push((
                    params.clone(),
                    guard.clone(),
                    body.clone(),
                ));
            }
            Stmt::FnDefGuardedTyped(name, params, ret_ann, guard, body) => {
                env.fn_types.insert(
                    name.clone(),
                    (
                        params.iter().map(|(_, t)| t.clone()).collect(),
                        ret_ann.clone(),
                    ),
                );
                env.guarded_funcs.entry(name.clone()).or_default().push((
                    params.iter().map(|(n, _)| n.clone()).collect(),
                    guard.clone(),
                    body.clone(),
                ));
            }
            Stmt::FnDefPattern(name, patterns, guard, ret_ann, body) => {
                env.pattern_funcs.entry(name.clone()).or_default().push((
                    patterns.clone(),
                    guard.clone(),
                    ret_ann.clone(),
                    body.clone(),
                ));
            }
            Stmt::FnMain(body) => {
                env.funcs
                    .insert("main".to_string(), (Vec::new(), body.clone()));
            }
            Stmt::DataDecl(_, ctors) => {
                for c in ctors {
                    env.data_ctors.insert(c.name.clone(), c.arity);
                }
            }
            _ => {}
        }
    }
    // Execute top-level statements (import/let/assign/print)
    for stmt in &program.stmts {
        match stmt {
            Stmt::Import(name, module) => {
                let v = env.import_module(&module)?;
                env.vars_mut().insert(name.clone(), v);
            }
            Stmt::Let(name, expr) => {
                let v = env.eval_expr(expr.clone())?;
                env.declare_let(name.clone(), v);
            }
            Stmt::Var(name, expr) => {
                let v = env.eval_expr(expr.clone())?;
                env.declare_var(name.clone(), v);
            }
            Stmt::Assign(name, expr) => {
                let v = env.eval_expr(expr.clone())?;
                env.assign_var(&name, v)?;
            }
            Stmt::ArrayAssign(name, idx_expr, val_expr) => {
                let idx_v = env.eval_expr(idx_expr.clone())?;
                let new_v = env.eval_expr(val_expr.clone())?;
                match idx_v {
                    Value::Int(i) => env.array_assign(&name, i, new_v)?,
                    _ => bail!("array element assignment expects array variable and int index"),
                }
            }
            Stmt::LetTuple(names, expr) => {
                let v = env.eval_expr(expr.clone())?;
                match v {
                    Value::Array(items) => {
                        if items.len() != names.len() {
                            bail!("tuple arity mismatch");
                        }
                        for (n, it) in names.iter().zip(items.into_iter()) {
                            env.declare_let(n.clone(), it);
                        }
                    }
                    _ => bail!("destructuring expects array value"),
                }
            }
            Stmt::Print(args) => {
                let parts: Result<Vec<String>> = args
                    .iter()
                    .cloned()
                    .map(|e| env.eval_expr(e).map(|v| format_value(&v)))
                    .collect();
                println!("{}", parts?.join(" "));
            }
            Stmt::Expr(e) => {
                let _ = env.eval_expr(e.clone())?;
            }
            Stmt::DataDecl(type_name, ctors) => {
                let mut names = Vec::new();
                for c in ctors {
                    env.data_ctors.insert(c.name.clone(), c.arity);
                    names.push(c.name.clone());
                }
                env.type_ctors.insert(type_name.clone(), names);
            }
            Stmt::FnDefGuarded(_, _, _, _) => { /* already collected */ }
            Stmt::FnDefGuardedTyped(_, _, _, _, _) => { /* already collected */ }
            // Allow control-flow at top-level
            Stmt::If(..) | Stmt::While(..) | Stmt::Parallel(..) | Stmt::Return(..) => {
                let _ = env.exec_stmt(&stmt)?;
            }
            _ => {}
        }
    }

    Ok(())
}