#[cfg(feature = "sdl3")]
use anyhow::anyhow; // anyhow! macro for SDL code paths
use anyhow::{bail, Result, anyhow};
use std::collections::HashMap;
#[cfg(feature = "sdl3")]
use std::time::Duration; // Duration used in sdl_delay

use crate::parser::{BinOp, Expr, Pattern, Program, Stmt};

// Clause type aliases to keep signatures and storage readable (avoid clippy::type_complexity)
type GuardedClause = (Vec<String>, Expr, Vec<Stmt>);
type PatternClause = (Vec<Pattern>, Option<Expr>, Vec<Stmt>);

// Built-in module registry (update when adding more modules)
pub fn builtin_modules() -> Vec<&'static str> {
    vec!["sdl", "linalg"]
}

// Core built-in functions (non-module) recognized directly by the evaluator.
// Keep in sync with match arms in Expr::Call in eval_expr.
pub fn builtin_functions() -> Vec<&'static str> {
    // print is a statement in the AST but users expect it; include for discoverability.
    let mut v = vec!["print", "len", "sum", "filter", "reduce", "map", "import"];
    v.sort();
    v
}

pub fn execute(program: Program, debug: bool) -> Result<()> {
    let mut env = Env::new();
    env.debug = debug;
    if env.debug { eprintln!("[TONG][dbg] start: {} top-level statements", program.stmts.len()); }

    // First collect function definitions
    for (i, stmt) in program.stmts.iter().enumerate() {
        if env.debug { eprintln!("[TONG][dbg] top-level stmt #{}: {}", i, stmt.kind_name()); }
        match stmt {
            Stmt::FnDef(name, params, body) => {
                env.funcs
                    .insert(name.clone(), (params.clone(), body.clone()));
            }
            Stmt::FnDefGuarded(name, params, guard, body) => {
                env.guarded_funcs.entry(name.clone()).or_default().push((
                    params.clone(),
                    guard.clone(),
                    body.clone(),
                ));
            }
            Stmt::FnDefPattern(name, patterns, guard, body) => {
                env.pattern_funcs.entry(name.clone()).or_default().push((
                    patterns.clone(),
                    guard.clone(),
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
        for (idx, (pats, guard, _body)) in clauses.iter().enumerate() {
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
        let mut seen: Vec<(String, usize)> = Vec::new();
        for (idx, (pats, guard, _)) in clauses.iter().enumerate() {
            if guard.is_some() {
                continue; // Guards may differentiate runtime reachability; skip for now
            }
            let key = pats.iter().map(pat_key).collect::<Vec<_>>().join("|");
            if let Some((_, prev_idx)) = seen.iter().find(|(k, _)| k == &key) {
                if std::env::var("TONG_NO_MATCH_WARN").is_err() {
                    eprintln!(
                        "[TONG][warn] redundant pattern function clause #{} for '{}' (covered by earlier clause #{})",
                        idx, fname, prev_idx
                    );
                }
            } else {
                seen.push((key, idx));
            }
        }
    }

    // Execute top-level statements (import/let/assign/print)
    for stmt in &program.stmts {
        match stmt {
            Stmt::Import(name, module) => {
                let v = env.import_module(module)?;
                env.vars_mut().insert(name.clone(), v);
            }
            Stmt::Let(name, expr) => {
                let v = env.eval_expr(expr.clone())?;
                env.vars_mut().insert(name.clone(), v);
            }
            Stmt::Assign(name, expr) => {
                let v = env.eval_expr(expr.clone())?;
                env.vars_mut().insert(name.clone(), v);
            }
            Stmt::ArrayAssign(name, idx_expr, val_expr) => {
                let base = env.get_var(name).ok_or_else(|| anyhow!(format!("undefined variable {}", name)))?;
                let idx_v = env.eval_expr(idx_expr.clone())?;
                let new_v = env.eval_expr(val_expr.clone())?;
                match (base, idx_v) {
                    (Value::Array(mut items), Value::Int(i)) => {
                        if i < 0 { bail!("negative index") }
                        let ui = i as usize;
                        if ui >= items.len() { bail!("index out of bounds") }
                        items[ui] = new_v;
                        env.vars_mut().insert(name.clone(), Value::Array(items));
                    }
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
                            env.vars_mut().insert(n.clone(), it);
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
            _ => {}
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum Value {
    Str(String),
    Float(f64),
    Int(i64),
    Bool(bool),
    Array(Vec<Value>),
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
        env: HashMap<String, Value>,
    },
    FuncRef(String),
    Object(HashMap<String, Value>),
    Constructor {
        name: String,
        fields: Vec<Value>,
    },
    Partial {
        name: String,
        applied: Vec<Value>,
    },
}

#[derive(Default)]
struct Env {
    vars_stack: Vec<HashMap<String, Value>>, // lexical-style stack
    funcs: HashMap<String, (Vec<String>, Vec<Stmt>)>,
    guarded_funcs: HashMap<String, Vec<GuardedClause>>, // guarded multi-clause
    pattern_funcs: HashMap<String, Vec<PatternClause>>, // pattern parameter clauses
    modules: HashMap<String, Value>,
    #[cfg(not(feature = "sdl3"))]
    sdl_frame: i64,
    #[cfg(feature = "sdl3")]
    sdl: Option<SdlState>,
    data_ctors: HashMap<String, usize>,       // ctor name -> arity
    type_ctors: HashMap<String, Vec<String>>, // type name -> ctor names
    ctor_type: HashMap<String, String>,       // ctor name -> type name
    debug: bool,
}

impl Env {
    // Execute a sequence of statements. Return Some(value) if a Return was hit.
    fn exec_block(&mut self, block: &[Stmt]) -> Result<Option<Value>> {
        let mut last_expr: Option<Value> = None;
        for s in block {
            match self.exec_stmt(s)? {
                Some(rv) => return Ok(Some(rv)), // explicit return short-circuits
                None => {
                    // Capture value of bare expression statements for implicit return semantics
                    if let Stmt::Expr(e) = s {
                        // Evaluate again to capture value (exec_stmt already evaluated it and discarded). To avoid double eval, we could special-case in exec_stmt, but keep simple for now.
                        last_expr = Some(self.eval_expr(e.clone())?);
                    }
                }
            }
        }
        // If no explicit return, propagate last expression value (functional style implicit return)
        Ok(last_expr)
    }

    // Execute a single statement. Return Some(value) if a Return was hit.
    fn exec_stmt(&mut self, s: &Stmt) -> Result<Option<Value>> {
        if self.debug { eprintln!("[TONG][dbg] exec {:?}", s.kind_name()); }
        match s {
            Stmt::Import(n, m) => {
                let v = self.import_module(m)?;
                self.vars_mut().insert(n.clone(), v);
                Ok(None)
            }
            Stmt::Let(n, e) => {
                let v = self.eval_expr(e.clone())?;
                self.vars_mut().insert(n.clone(), v);
                Ok(None)
            }
            Stmt::Assign(n, e) => {
                let v = self.eval_expr(e.clone())?;
                self.vars_mut().insert(n.clone(), v);
                Ok(None)
            }
            Stmt::ArrayAssign(name, idx_expr, val_expr) => {
                let base = self.get_var(name).ok_or_else(|| anyhow!(format!("undefined variable {}", name)))?;
                let idx_v = self.eval_expr(idx_expr.clone())?;
                let new_v = self.eval_expr(val_expr.clone())?;
                match (base, idx_v) {
                    (Value::Array(mut items), Value::Int(i)) => {
                        if i < 0 { bail!("negative index") }
                        let ui = i as usize;
                        if ui >= items.len() { bail!("index out of bounds") }
                        items[ui] = new_v;
                        self.vars_mut().insert(name.clone(), Value::Array(items));
                        Ok(None)
                    }
                    _ => bail!("array element assignment expects array variable and int index"),
                }
            }
            Stmt::Print(args) => {
                let mut parts = Vec::new();
                for e in args {
                    parts.push(format_value(&self.eval_expr(e.clone())?));
                }
                println!("{}", parts.join(" "));
                Ok(None)
            }
            Stmt::Return(e) => {
                let v = self.eval_expr(e.clone())?;
                Ok(Some(v))
            }
            Stmt::FnDef(name, params, body) => {
                self.funcs
                    .insert(name.clone(), (params.clone(), body.clone()));
                Ok(None)
            }
            Stmt::FnDefGuarded(name, params, guard, body) => {
                self.guarded_funcs.entry(name.clone()).or_default().push((
                    params.clone(),
                    guard.clone(),
                    body.clone(),
                ));
                Ok(None)
            }
            Stmt::FnDefPattern(name, patterns, guard, body) => {
                self.pattern_funcs.entry(name.clone()).or_default().push((
                    patterns.clone(),
                    guard.clone(),
                    body.clone(),
                ));
                Ok(None)
            }
            Stmt::LetTuple(names, expr) => {
                let v = self.eval_expr(expr.clone())?;
                match v {
                    Value::Array(items) => {
                        if items.len() != names.len() {
                            bail!("tuple arity mismatch");
                        }
                        for (n, it) in names.iter().zip(items.into_iter()) {
                            self.vars_mut().insert(n.clone(), it);
                        }
                        Ok(None)
                    }
                    _ => bail!("destructuring expects array value"),
                }
            }
            Stmt::DataDecl(type_name, ctors) => {
                let mut names = Vec::new();
                for c in ctors {
                    self.data_ctors.insert(c.name.clone(), c.arity);
                    names.push(c.name.clone());
                    self.ctor_type.insert(c.name.clone(), type_name.clone());
                }
                self.type_ctors.insert(type_name.clone(), names);
                Ok(None)
            }
            Stmt::Expr(e) => {
                let _ = self.eval_expr(e.clone())?;
                Ok(None)
            }
            Stmt::FnMain(_) => Ok(None),
            Stmt::If(cond, then_body, else_body) => {
                let v = self.eval_expr(cond.clone())?;
                if matches!(v, Value::Bool(true)) {
                    self.exec_block(then_body)
                } else if let Some(eb) = else_body {
                    self.exec_block(eb)
                } else {
                    Ok(None)
                }
            }
            Stmt::While(cond, body) => {
                loop {
                    let v = self.eval_expr(cond.clone())?;
                    if !matches!(v, Value::Bool(true)) { break; }
                    // Execute statements but ignore implicit last expression value to prevent accidental loop termination
                    for s in body {
                        if let Some(rv) = self.exec_stmt(s)? { return Ok(Some(rv)); }
                    }
                }
                Ok(None)
            }
            Stmt::Parallel(inner) => {
                for is in inner {
                    if let Some(rv) = self.exec_stmt(is)? {
                        return Ok(Some(rv));
                    }
                }
                Ok(None)
            }
        }
    }
    fn new() -> Self {
        Self {
            vars_stack: vec![HashMap::new()],
            funcs: HashMap::new(),
            guarded_funcs: HashMap::new(),
            pattern_funcs: HashMap::new(),
            modules: HashMap::new(),
            #[cfg(not(feature = "sdl3"))]
            sdl_frame: 0,
            #[cfg(feature = "sdl3")]
            sdl: None,
            data_ctors: HashMap::new(),
            type_ctors: HashMap::new(),
            ctor_type: HashMap::new(),
            debug: false,
        }
    }
    fn vars(&self) -> &HashMap<String, Value> {
        self.vars_stack.last().unwrap()
    }
    fn vars_mut(&mut self) -> &mut HashMap<String, Value> {
        self.vars_stack.last_mut().unwrap()
    }
    fn get_var(&self, name: &str) -> Option<Value> {
        for frame in self.vars_stack.iter().rev() {
            if let Some(v) = frame.get(name) {
                return Some(v.clone());
            }
        }
        None
    }

    fn eval_expr(&mut self, e: Expr) -> Result<Value> {
        let v = match e {
            Expr::Property { target, name } => {
                let obj = self.eval_expr(*target)?;
                match obj {
                    Value::Object(map) => map
                        .get(&name)
                        .cloned()
                        .ok_or_else(|| anyhow::anyhow!(format!("unknown property {}", name)))?,
                    _ => bail!("property access on non-object"),
                }
            }
            Expr::MethodCall {
                target,
                method,
                args,
            } => {
                let obj_val = self.eval_expr(*target)?;
                match obj_val.clone() {
                    Value::Object(map) => {
                        let func = map
                            .get(&method)
                            .cloned()
                            .ok_or_else(|| anyhow::anyhow!("unknown method {}", method))?;
                        let mut values: Vec<Value> = Vec::new();
                        // For our module methods, we pass evaluated args; some APIs expect (handle, ...), but our API encodes handle as first arg explicitly in the .tong code.
                        for a in args {
                            values.push(self.eval_expr(a)?);
                        }
                        match func {
                            Value::FuncRef(name) => {
                                if name.starts_with("sdl_") {
                                    // Build Expr args: if the first arg is a renderer/window handle placeholder, map to a simple int; otherwise, just map basic values.
                                    let expr_args: Vec<Expr> =
                                        values.into_iter().map(|v| expr_from_value(&v)).collect();
                                    self.call_sdl_builtin(&name, expr_args)?
                                } else if name.starts_with("linalg_") {
                                    self.call_linalg_builtin_values(&name, values)?
                                } else {
                                    self.call_function_values(name, values)?
                                }
                            }
                            Value::Lambda { params, body, env } => {
                                self.call_lambda_values(params, *body, env, values)?
                            }
                            _ => bail!("{} is not callable", method),
                        }
                    }
                    _ => bail!("method call on non-object"),
                }
            }
            Expr::Lambda { params, body } => {
                // capture current top frame
                let env = self.vars().clone();
                Value::Lambda { params, body, env }
            }
            Expr::Str(s) => Value::Str(s),
            Expr::Float(f) => Value::Float(f),
            Expr::Int(i) => Value::Int(i),
            Expr::Bool(b) => Value::Bool(b),
            Expr::UnaryNeg(inner) => match self.eval_expr(*inner)? {
                Value::Int(i) => Value::Int(-i),
                Value::Float(f) => Value::Float(-f),
                _ => bail!("unary '-' expects numeric"),
            },
            Expr::UnaryNot(inner) => match self.eval_expr(*inner)? {
                Value::Bool(b) => Value::Bool(!b),
                _ => bail!("unary '!' expects Bool"),
            },
            Expr::Ident(name) => {
                if let Some(v) = self.get_var(&name) {
                    v
                } else if self.funcs.contains_key(&name) || self.pattern_funcs.contains_key(&name) {
                    Value::FuncRef(name)
                } else if let Some(arity) = self.data_ctors.get(&name) {
                    if *arity == 0 {
                        Value::Constructor {
                            name,
                            fields: vec![],
                        }
                    } else {
                        Value::FuncRef(name)
                    }
                } else {
                    bail!("undefined variable {}", name)
                }
            }
            Expr::Array(items) => {
                let mut out = Vec::new();
                for it in items {
                    out.push(self.eval_expr(it)?);
                }
                Value::Array(out)
            }
            Expr::Index(target, idx) => {
                let arr = self.eval_expr(*target)?;
                let i = self.eval_expr(*idx)?;
                match (arr, i) {
                    (Value::Array(v), Value::Int(n)) => {
                        if n < 0 {
                            bail!("negative index not supported")
                        };
                        let ni = n as usize;
                        v.get(ni)
                            .cloned()
                            .ok_or_else(|| anyhow::anyhow!("index out of bounds"))?
                    }
                    _ => bail!("indexing expects array[index]"),
                }
            }
            Expr::ConstructorCall { name, args } => {
                let arity = *self
                    .data_ctors
                    .get(&name)
                    .ok_or_else(|| anyhow::anyhow!(format!("unknown constructor {}", name)))?;
                let evaled: Result<Vec<Value>> =
                    args.into_iter().map(|a| self.eval_expr(a)).collect();
                let vals = evaled?;
                if vals.len() < arity {
                    Value::Partial {
                        name,
                        applied: vals,
                    }
                } else if vals.len() == arity {
                    Value::Constructor { name, fields: vals }
                } else {
                    bail!("constructor arity mismatch")
                }
            }
            Expr::Call { callee, args } => {
                match callee.as_str() {
                    "len" => {
                        if args.len() != 1 {
                            bail!("len expects 1 argument");
                        }
                        let v0 = self.eval_expr(args[0].clone())?;
                        match v0 {
                            Value::Array(v) => Value::Int(v.len() as i64),
                            _ => bail!("len expects array"),
                        }
                    }
                    "import" => {
                        if args.len() != 1 {
                            bail!("import expects 1 argument");
                        }
                        let v0 = self.eval_expr(args[0].clone())?;
                        match v0 {
                            Value::Str(name) => self.import_module(&name)?,
                            _ => bail!("import expects string module name"),
                        }
                    }
                    "sum" => {
                        if args.len() != 1 {
                            bail!("sum expects 1 argument");
                        }
                        let v0 = self.eval_expr(args[0].clone())?;
                        match v0 {
                            Value::Array(v) => {
                                let mut is_float = false;
                                let mut total_f = 0.0f64;
                                let mut total_i: i64 = 0;
                                for it in v {
                                    match it {
                                        Value::Int(i) => total_i += i,
                                        Value::Float(f) => {
                                            total_f += f;
                                            is_float = true;
                                        }
                                        _ => bail!("sum expects numeric array"),
                                    }
                                }
                                if is_float {
                                    Value::Float(total_i as f64 + total_f)
                                } else {
                                    Value::Int(total_i)
                                }
                            }
                            _ => bail!("sum expects array"),
                        }
                    }
                    "filter" => {
                        if args.len() != 2 {
                            bail!("filter expects 2 arguments (array, function)");
                        }
                        let arr_val = self.eval_expr(args[0].clone())?;
                        let callable = args[1].clone();
                        match arr_val {
                            Value::Array(items) => {
                                let mut out = Vec::new();
                                for item in items {
                                    let res = self.apply_callable(
                                        callable.clone(),
                                        vec![expr_from_value(&item)],
                                    )?;
                                    match res {
                                        Value::Bool(true) => out.push(item),
                                        Value::Bool(false) => {}
                                        _ => bail!("filter function must return bool"),
                                    }
                                }
                                Value::Array(out)
                            }
                            _ => bail!("filter expects array as first argument"),
                        }
                    }
                    "reduce" => {
                        if args.len() != 3 {
                            bail!("reduce expects 3 arguments (array, function, initial)");
                        }
                        let arr_val = self.eval_expr(args[0].clone())?;
                        let callable = args[1].clone();
                        let mut acc = self.eval_expr(args[2].clone())?;
                        match arr_val {
                            Value::Array(items) => {
                                for item in items {
                                    acc = self.apply_callable(
                                        callable.clone(),
                                        vec![expr_from_value(&acc), expr_from_value(&item)],
                                    )?;
                                }
                                acc
                            }
                            _ => bail!("reduce expects array as first argument"),
                        }
                    }
                    "map" => {
                        if args.len() != 2 {
                            bail!("map expects 2 arguments (array, function)");
                        }
                        let arr_val = self.eval_expr(args[0].clone())?;
                        let callable = args[1].clone();
                        match arr_val {
                            Value::Array(items) => {
                                let mut out = Vec::new();
                                for item in items {
                                    let arg_expr = expr_from_value(&item);
                                    let v =
                                        self.apply_callable(callable.clone(), vec![arg_expr])?;
                                    out.push(v);
                                }
                                Value::Array(out)
                            }
                            _ => bail!("map expects array as first argument"),
                        }
                    }
                    _ => {
                        // user-defined function: evaluate all args and call
                        // First, see if there is a variable with this name (could be a lambda or function ref)
                        if let Some(v) = self.get_var(&callee) {
                            match v {
                                Value::Lambda { params, body, env } => {
                                    let evaled: Result<Vec<Value>> =
                                        args.iter().cloned().map(|a| self.eval_expr(a)).collect();
                                    let vals = evaled?;
                                    if vals.len() < params.len() {
                                        // build new lambda with remaining params capturing applied ones
                                        let mut captured = env.clone();
                                        for (p, vv) in params.iter().zip(vals.iter()) {
                                            captured.insert(p.clone(), vv.clone());
                                        }
                                        let remaining = params[vals.len()..].to_vec();
                                        Value::Lambda {
                                            params: remaining,
                                            body: body.clone(),
                                            env: captured,
                                        }
                                    } else if vals.len() == params.len() {
                                        self.call_lambda_values(params, *body, env, vals)?
                                    } else {
                                        bail!("too many arguments for lambda")
                                    }
                                }
                                Value::FuncRef(name) => {
                                    let evaled: Result<Vec<Value>> =
                                        args.into_iter().map(|a| self.eval_expr(a)).collect();
                                    self.call_function_values(name, evaled?)?
                                }
                                Value::Partial { name, applied } => {
                                    // extend partial
                                    let evaled: Result<Vec<Value>> =
                                        args.into_iter().map(|a| self.eval_expr(a)).collect();
                                    let mut new_applied = applied.clone();
                                    new_applied.extend(evaled?);
                                    // Determine target arity (function or constructor)
                                    if let Some((params, _)) = self.funcs.get(&name) {
                                        if new_applied.len() < params.len() {
                                            Value::Partial {
                                                name,
                                                applied: new_applied,
                                            }
                                        } else if new_applied.len() == params.len() {
                                            self.call_function_values(name, new_applied)?
                                        } else {
                                            bail!("too many arguments for function partial")
                                        }
                                    } else if let Some(arity) = self.data_ctors.get(&name) {
                                        if new_applied.len() < *arity {
                                            Value::Partial {
                                                name,
                                                applied: new_applied,
                                            }
                                        } else if new_applied.len() == *arity {
                                            Value::Constructor {
                                                name,
                                                fields: new_applied,
                                            }
                                        } else {
                                            bail!("too many arguments for constructor partial")
                                        }
                                    } else {
                                        bail!("unknown target in partial")
                                    }
                                }
                                _ => bail!("{} is not callable", callee),
                            }
                        } else if self.funcs.contains_key(&callee) {
                            let evaled: Result<Vec<Value>> =
                                args.into_iter().map(|a| self.eval_expr(a)).collect();
                            let vals = evaled?;
                            let (params, _) = self.funcs.get(&callee).cloned().unwrap();
                            if vals.len() < params.len() {
                                Value::Partial {
                                    name: callee.clone(),
                                    applied: vals,
                                }
                            } else if vals.len() == params.len() {
                                self.call_function_values(callee.clone(), vals)?
                            } else {
                                bail!("too many arguments")
                            }
                        } else if self.guarded_funcs.contains_key(&callee) {
                            let evaled: Result<Vec<Value>> =
                                args.into_iter().map(|a| self.eval_expr(a)).collect();
                            let vals = evaled?;
                            let clauses = self.guarded_funcs.get(&callee).unwrap();
                            let arity = clauses.first().map(|c| c.0.len()).unwrap_or(0);
                            if vals.len() < arity {
                                Value::Partial {
                                    name: callee.clone(),
                                    applied: vals,
                                }
                            } else if vals.len() == arity {
                                self.call_guarded_function_values(callee.clone(), vals)?
                            } else {
                                bail!("too many arguments")
                            }
                        } else if self.pattern_funcs.contains_key(&callee) {
                            let evaled: Result<Vec<Value>> =
                                args.into_iter().map(|a| self.eval_expr(a)).collect();
                            let vals = evaled?;
                            let clauses = self.pattern_funcs.get(&callee).unwrap();
                            let arity = clauses.first().map(|c| c.0.len()).unwrap_or(0);
                            if vals.len() < arity {
                                Value::Partial {
                                    name: callee.clone(),
                                    applied: vals,
                                }
                            } else if vals.len() == arity {
                                self.call_pattern_function_values(callee.clone(), vals)?
                            } else {
                                bail!("too many arguments")
                            }
                        } else if let Some(arity) = self.data_ctors.get(&callee).cloned() {
                            let evaled: Result<Vec<Value>> =
                                args.into_iter().map(|a| self.eval_expr(a)).collect();
                            let vals = evaled?;
                            if vals.len() < arity {
                                Value::Partial {
                                    name: callee,
                                    applied: vals,
                                }
                            } else if vals.len() == arity {
                                Value::Constructor {
                                    name: callee,
                                    fields: vals,
                                }
                            } else {
                                bail!("constructor arity mismatch")
                            }
                        } else if callee == "<partial>" {
                            bail!("invalid partial placeholder")
                        } else if self.guarded_funcs.contains_key(&callee) {
                            let evaled: Result<Vec<Value>> =
                                args.into_iter().map(|a| self.eval_expr(a)).collect();
                            let vals = evaled?;
                            // Determine arity from first clause
                            let clauses = self.guarded_funcs.get(&callee).unwrap();
                            let arity = clauses.first().map(|c| c.0.len()).unwrap_or(0);
                            if vals.len() < arity {
                                Value::Partial {
                                    name: callee.clone(),
                                    applied: vals,
                                }
                            } else if vals.len() == arity {
                                self.call_guarded_function_values(callee.clone(), vals)?
                            } else {
                                bail!("too many arguments")
                            }
                        } else {
                            bail!("unknown function {}", callee)
                        }
                    }
                }
            }
            Expr::ListComp {
                elem,
                generators,
                pred,
            } => {
                // Recursive helper to bind generators left-to-right
                fn eval_gens(
                    env: &mut Env,
                    gens: &[(String, Expr)],
                    idx: usize,
                    elem: &Expr,
                    pred: &Option<Box<Expr>>,
                    out: &mut Vec<Value>,
                ) -> Result<()> {
                    if idx == gens.len() {
                        // All generators bound; evaluate predicate then elem
                        if let Some(p) = pred {
                            let pv = env.eval_expr(*p.clone())?;
                            if !matches!(pv, Value::Bool(true)) {
                                return Ok(());
                            }
                        }
                        let ev = env.eval_expr(elem.clone())?;
                        out.push(ev);
                        return Ok(());
                    }
                    let (var, list_expr) = &gens[idx];
                    let list_val = env.eval_expr(list_expr.clone())?;
                    match list_val {
                        Value::Array(items) => {
                            for it in items {
                                // push new scope for each binding to avoid leaking between iterations
                                env.vars_stack.push(HashMap::new());
                                env.vars_mut().insert(var.clone(), it);
                                eval_gens(env, gens, idx + 1, elem, pred, out)?;
                                env.vars_stack.pop();
                            }
                            Ok(())
                        }
                        _ => bail!(
                            "list comprehension expects array source for generator '{}'",
                            var
                        ),
                    }
                }
                let mut out = Vec::new();
                eval_gens(self, &generators, 0, &elem, &pred, &mut out)?;
                Value::Array(out)
            }
            Expr::Block(stmts) => {
                self.vars_stack.push(HashMap::new());
                let mut last: Option<Value> = None;
                for s in stmts {
                    match self.exec_stmt(&s)? {
                        Some(rv) => { last = Some(rv); break; }
                        None => {
                            if let Stmt::Expr(e) = s.clone() {
                                // capture expression value
                                last = Some(self.eval_expr(e)?);
                            }
                        }
                    }
                }
                self.vars_stack.pop();
                last.unwrap_or(Value::Array(vec![]))
            }
            Expr::Match { scrutinee, arms } => {
                // Run redundancy analysis before executing match
                self.check_match_redundancy(&arms);
                let val = self.eval_expr(*scrutinee.clone())?;
                for (pat, guard, body) in arms.clone() {
                    self.vars_stack.push(HashMap::new());
                    let matched = self.match_pattern(&pat, &val)?;
                    let guard_pass = if matched {
                        if let Some(g) = guard.clone() {
                            matches!(self.eval_expr(g)?, Value::Bool(true))
                        } else {
                            true
                        }
                    } else {
                        false
                    };
                    if matched && guard_pass {
                        let res = self.eval_expr(body)?;
                        self.vars_stack.pop();
                        // crude exhaustiveness check: if no wildcard and scrutinee is a constructor with a known type, warn if uncovered ctors remain
                        self.check_match_exhaustiveness(&scrutinee, &arms)?;
                        return Ok(res);
                    }
                    self.vars_stack.pop();
                }
                // no arm matched
                eprintln!("[TONG][warn] non-exhaustive match at runtime");
                bail!("non-exhaustive match")
            }
            Expr::Binary { op, left, right } => {
                // Short-circuit for logical AND
                if let BinOp::And = op {
                    let l = self.eval_expr(*left)?;
                    match l {
                        Value::Bool(false) => Value::Bool(false), // don't eval right
                        Value::Bool(true) => {
                            let r = self.eval_expr(*right)?;
                            match r {
                                Value::Bool(b) => Value::Bool(b),
                                _ => bail!("right operand of '&' must be Bool"),
                            }
                        }
                        _ => bail!("left operand of '&' must be Bool"),
                    }
                } else if let BinOp::Or = op {
                    let l = self.eval_expr(*left)?;
                    match l {
                        Value::Bool(true) => Value::Bool(true), // short-circuit
                        Value::Bool(false) => {
                            let r = self.eval_expr(*right)?;
                            match r {
                                Value::Bool(b) => Value::Bool(b),
                                _ => bail!("right operand of '||' must be Bool"),
                            }
                        }
                        _ => bail!("left operand of '||' must be Bool"),
                    }
                } else {
                    let l = self.eval_expr(*left)?;
                    let r = self.eval_expr(*right)?;
                    match (l, r, op) {
                    // Logical AND: both operands must be Bool; left short-circuit implemented earlier by evaluating sequentially (we already evaluated r, so mimic semantics w/o side effects distinction). For true short-circuit we need special case before evaluating r; adjust above.
                    (Value::Bool(a), Value::Bool(b), BinOp::And) => Value::Bool(a && b),
                    (Value::Int(a), Value::Int(b), BinOp::Add) => Value::Int(a + b),
                    (Value::Int(a), Value::Int(b), BinOp::Sub) => Value::Int(a - b),
                    (Value::Int(a), Value::Int(b), BinOp::Mul) => Value::Int(a * b),
                    (Value::Int(a), Value::Int(b), BinOp::Div) => Value::Float(a as f64 / b as f64),
                    (Value::Int(a), Value::Int(b), BinOp::Mod) => Value::Int(a % b),

                    (Value::Float(a), Value::Float(b), BinOp::Add) => Value::Float(a + b),
                    (Value::Float(a), Value::Float(b), BinOp::Sub) => Value::Float(a - b),
                    (Value::Float(a), Value::Float(b), BinOp::Mul) => Value::Float(a * b),
                    (Value::Float(a), Value::Float(b), BinOp::Div) => Value::Float(a / b),

                    // int/float mixed
                    (Value::Int(a), Value::Float(b), BinOp::Add) => Value::Float(a as f64 + b),
                    (Value::Int(a), Value::Float(b), BinOp::Sub) => Value::Float(a as f64 - b),
                    (Value::Int(a), Value::Float(b), BinOp::Mul) => Value::Float(a as f64 * b),
                    (Value::Int(a), Value::Float(b), BinOp::Div) => Value::Float(a as f64 / b),
                    (Value::Float(a), Value::Int(b), BinOp::Add) => Value::Float(a + b as f64),
                    (Value::Float(a), Value::Int(b), BinOp::Sub) => Value::Float(a - b as f64),
                    (Value::Float(a), Value::Int(b), BinOp::Mul) => Value::Float(a * b as f64),
                    (Value::Float(a), Value::Int(b), BinOp::Div) => Value::Float(a / b as f64),

                    (Value::Float(a), Value::Float(b), BinOp::Eq) => {
                        Value::Bool((a - b).abs() < f64::EPSILON)
                    }
                    (Value::Int(a), Value::Int(b), BinOp::Eq) => Value::Bool(a == b),
                    (Value::Bool(a), Value::Bool(b), BinOp::Eq) => Value::Bool(a == b),
                    (Value::Str(a), Value::Str(b), BinOp::Eq) => Value::Bool(a == b),

                    (Value::Float(a), Value::Float(b), BinOp::Ne) => {
                        Value::Bool((a - b).abs() >= f64::EPSILON)
                    }
                    (Value::Int(a), Value::Int(b), BinOp::Ne) => Value::Bool(a != b),
                    (Value::Bool(a), Value::Bool(b), BinOp::Ne) => Value::Bool(a != b),
                    (Value::Str(a), Value::Str(b), BinOp::Ne) => Value::Bool(a != b),

                    (Value::Int(a), Value::Int(b), BinOp::Lt) => Value::Bool(a < b),
                    (Value::Int(a), Value::Int(b), BinOp::Le) => Value::Bool(a <= b),
                    (Value::Int(a), Value::Int(b), BinOp::Gt) => Value::Bool(a > b),
                    (Value::Int(a), Value::Int(b), BinOp::Ge) => Value::Bool(a >= b),
                    (Value::Float(a), Value::Float(b), BinOp::Lt) => Value::Bool(a < b),
                    (Value::Float(a), Value::Float(b), BinOp::Le) => Value::Bool(a <= b),
                    (Value::Float(a), Value::Float(b), BinOp::Gt) => Value::Bool(a > b),
                    (Value::Float(a), Value::Float(b), BinOp::Ge) => Value::Bool(a >= b),
                    (Value::Float(a), Value::Int(b), BinOp::Lt) => Value::Bool(a < b as f64),
                    (Value::Float(a), Value::Int(b), BinOp::Le) => Value::Bool(a <= b as f64),
                    (Value::Float(a), Value::Int(b), BinOp::Gt) => Value::Bool(a > b as f64),
                    (Value::Float(a), Value::Int(b), BinOp::Ge) => Value::Bool(a >= b as f64),
                    (Value::Int(a), Value::Float(b), BinOp::Lt) => Value::Bool((a as f64) < b),
                    (Value::Int(a), Value::Float(b), BinOp::Le) => Value::Bool((a as f64) <= b),
                    (Value::Int(a), Value::Float(b), BinOp::Gt) => Value::Bool((a as f64) > b),
                    (Value::Int(a), Value::Float(b), BinOp::Ge) => Value::Bool((a as f64) >= b),

                        (l, r, _) => bail!("unsupported operands for operation: {:?}", (l, r)),
                    }
                }
            }
        };
        Ok(v)
    }

    fn apply_callable(&mut self, func: Expr, args: Vec<Expr>) -> Result<Value> {
        match func {
            Expr::Ident(name) => {
                // could be function name or variable holding function/lambda
                if let Some(v) = self.get_var(&name) {
                    match v {
                        Value::Lambda { params, body, env } => {
                            // Evaluate args then call with values to preserve object args
                            let evaled: Result<Vec<Value>> =
                                args.iter().cloned().map(|a| self.eval_expr(a)).collect();
                            let vals = evaled?;
                            if vals.len() < params.len() {
                                let mut captured = env.clone();
                                for (p, vv) in params.iter().zip(vals.iter()) {
                                    captured.insert(p.clone(), vv.clone());
                                }
                                let remaining = params[vals.len()..].to_vec();
                                Ok(Value::Lambda {
                                    params: remaining,
                                    body: body.clone(),
                                    env: captured,
                                })
                            } else if vals.len() == params.len() {
                                self.call_lambda_values(params, *body, env, vals)
                            } else {
                                bail!("too many arguments for lambda")
                            }
                        }
                        Value::FuncRef(fname) => {
                            let evaled: Result<Vec<Value>> =
                                args.into_iter().map(|a| self.eval_expr(a)).collect();
                            if self.funcs.contains_key(&fname) {
                                self.call_function_values(fname, evaled?)
                            } else if self.guarded_funcs.contains_key(&fname) {
                                self.call_guarded_function_values(fname, evaled?)
                            } else if self.pattern_funcs.contains_key(&fname) {
                                self.call_pattern_function_values(fname, evaled?)
                            } else {
                                bail!("unknown function {}", fname)
                            }
                        }
                        Value::Partial { name, applied } => {
                            let evaled: Result<Vec<Value>> =
                                args.into_iter().map(|a| self.eval_expr(a)).collect();
                            let mut new_applied = applied.clone();
                            new_applied.extend(evaled?);
                            if let Some((params, _)) = self.funcs.get(&name) {
                                if new_applied.len() < params.len() {
                                    Ok(Value::Partial {
                                        name,
                                        applied: new_applied,
                                    })
                                } else if new_applied.len() == params.len() {
                                    self.call_function_values(name, new_applied)
                                } else {
                                    bail!("too many arguments for function partial")
                                }
                            } else if let Some(clauses) = self.guarded_funcs.get(&name) {
                                let arity = clauses.first().map(|c| c.0.len()).unwrap_or(0);
                                if new_applied.len() < arity {
                                    Ok(Value::Partial {
                                        name,
                                        applied: new_applied,
                                    })
                                } else if new_applied.len() == arity {
                                    self.call_guarded_function_values(name, new_applied)
                                } else {
                                    bail!("too many arguments for function partial")
                                }
                            } else if let Some(patterns) = self.pattern_funcs.get(&name) {
                                let arity = patterns.first().map(|c| c.0.len()).unwrap_or(0);
                                if new_applied.len() < arity {
                                    Ok(Value::Partial {
                                        name,
                                        applied: new_applied,
                                    })
                                } else if new_applied.len() == arity {
                                    self.call_pattern_function_values(name, new_applied)
                                } else {
                                    bail!("too many arguments for function partial")
                                }
                            } else if let Some(arity) = self.data_ctors.get(&name) {
                                if new_applied.len() < *arity {
                                    Ok(Value::Partial {
                                        name,
                                        applied: new_applied,
                                    })
                                } else if new_applied.len() == *arity {
                                    Ok(Value::Constructor {
                                        name,
                                        fields: new_applied,
                                    })
                                } else {
                                    bail!("too many arguments for constructor partial")
                                }
                            } else {
                                bail!("unknown target in partial")
                            }
                        }
                        _ => bail!("{} is not callable", name),
                    }
                } else {
                    // delegate to call_function to handle built-ins (e.g., sdl_*) and user-defined functions
                    self.call_function(name, args)
                }
            }
            Expr::Lambda { params, body } => {
                let env = self.vars().clone();
                self.call_lambda(params, *body, env, args)
            }
            _ => bail!("callable must be a function name or lambda"),
        }
    }

    fn call_lambda(
        &mut self,
        params: Vec<String>,
        body: Expr,
        captured_env: HashMap<String, Value>,
        args: Vec<Expr>,
    ) -> Result<Value> {
        if params.len() != args.len() {
            bail!("arity mismatch for lambda");
        }
        // Push captured frame then a params frame
        self.vars_stack.push(captured_env);
        self.vars_stack.push(HashMap::new());
        for (p, a) in params.iter().zip(args.into_iter()) {
            let val = self.eval_expr(a)?;
            self.vars_mut().insert(p.clone(), val);
        }
        // Evaluate body expression
        let result = self.eval_expr(body.clone());
        // Pop frames
        self.vars_stack.pop();
        self.vars_stack.pop();
        result
    }

    // Lambda call with pre-evaluated values (avoids re-evaluating and supports object args)
    fn call_lambda_values(
        &mut self,
        params: Vec<String>,
        body: Expr,
        captured_env: HashMap<String, Value>,
        values: Vec<Value>,
    ) -> Result<Value> {
        if params.len() != values.len() {
            bail!("arity mismatch for lambda");
        }
        self.vars_stack.push(captured_env);
        self.vars_stack.push(HashMap::new());
        for (p, v) in params.iter().zip(values.into_iter()) {
            self.vars_mut().insert(p.clone(), v);
        }
        let result = self.eval_expr(body.clone());
        self.vars_stack.pop();
        self.vars_stack.pop();
        result
    }
    fn call_function(&mut self, name: String, args: Vec<Expr>) -> Result<Value> {
        if name.starts_with("sdl_") {
            return self.call_sdl_builtin(&name, args);
        }
        if name.starts_with("linalg_") {
            let evaled: Result<Vec<Value>> = args.into_iter().map(|a| self.eval_expr(a)).collect();
            return self.call_linalg_builtin_values(&name, evaled?);
        }
        if let Some((params, body)) = self.funcs.get(&name).cloned() {
            if params.len() != args.len() {
                bail!("arity mismatch for {}", name);
            }
            self.vars_stack.push(HashMap::new());
            for (p, a) in params.iter().zip(args.into_iter()) {
                let val = self.eval_expr(a)?;
                self.vars_mut().insert(p.clone(), val);
            }
            let ret = self.exec_block(&body)?;
            self.vars_stack.pop();
            Ok(ret.unwrap_or(Value::Int(0)))
        } else if let Some(clauses) = self.guarded_funcs.get(&name).cloned() {
            if clauses.is_empty() {
                bail!("no clauses for guarded function {name}");
            }
            let arity = clauses[0].0.len();
            if arity != args.len() {
                bail!("arity mismatch for {}", name);
            }
            let evaled: Result<Vec<Value>> = args.into_iter().map(|a| self.eval_expr(a)).collect();
            self.call_guarded_function_values(name, evaled?)
        } else if let Some(pattern_clauses) = self.pattern_funcs.get(&name).cloned() {
            // Determine arity from first clause's pattern count
            if let Some(first) = pattern_clauses.first() {
                let arity = first.0.len();
                if arity != args.len() {
                    bail!("arity mismatch for {}", name);
                }
                // Evaluate args once
                let evaled: Result<Vec<Value>> =
                    args.into_iter().map(|a| self.eval_expr(a)).collect();
                self.call_pattern_function_values(name, evaled?)
            } else {
                bail!("no pattern clauses for {}", name)
            }
        } else {
            bail!("unknown function {}", name)
        }
    }

    // Function call with pre-evaluated values (avoids expr_from_value conversion issues for objects)
    fn call_function_values(&mut self, name: String, values: Vec<Value>) -> Result<Value> {
        if name.starts_with("sdl_") {
            bail!("internal: call_function_values should not be used for SDL builtins");
        }
        if name.starts_with("linalg_") {
            return self.call_linalg_builtin_values(&name, values);
        }
        if let Some((params, body)) = self.funcs.get(&name).cloned() {
            if params.len() != values.len() {
                bail!("arity mismatch for {}", name);
            }
            self.vars_stack.push(HashMap::new());
            for (p, v) in params.iter().zip(values.into_iter()) {
                self.vars_mut().insert(p.clone(), v);
            }
            let ret = self.exec_block(&body)?;
            self.vars_stack.pop();
            Ok(ret.unwrap_or(Value::Int(0)))
        } else if let Some(clauses) = self.guarded_funcs.get(&name).cloned() {
            let arity = clauses.first().map(|c| c.0.len()).unwrap_or(0);
            if arity != values.len() {
                bail!("arity mismatch for {}", name);
            }
            self.call_guarded_function_values(name, values)
        } else if let Some(_clauses) = self.pattern_funcs.get(&name).cloned() {
            self.call_pattern_function_values(name, values)
        } else {
            bail!("unknown function {}", name)
        }
    }

    fn call_pattern_function_values(&mut self, name: String, values: Vec<Value>) -> Result<Value> {
        let clauses = self
            .pattern_funcs
            .get(&name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!(format!("unknown pattern function {name}")))?;
        for (patterns, guard, body) in clauses {
            if patterns.len() != values.len() {
                bail!("arity mismatch for {name}");
            }
            self.vars_stack.push(HashMap::new());
            let mut all_match = true;
            for (p, v) in patterns.iter().zip(values.iter()) {
                if !self.match_pattern(p, v)? {
                    all_match = false;
                    break;
                }
            }
            if all_match {
                let guard_ok = if let Some(gexpr) = guard.clone() {
                    matches!(self.eval_expr(gexpr)?, Value::Bool(true))
                } else {
                    true
                };
                if guard_ok {
                    let ret = self.exec_block(&body)?;
                    self.vars_stack.pop();
                    return Ok(ret.unwrap_or(Value::Int(0)));
                }
            }
            self.vars_stack.pop();
        }
        bail!("no pattern clause matched for {name}")
    }

    fn call_guarded_function_values(&mut self, name: String, values: Vec<Value>) -> Result<Value> {
        let clauses = self
            .guarded_funcs
            .get(&name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!(format!("unknown guarded function {name}")))?;
        for (params, guard_expr, body) in clauses {
            if params.len() != values.len() {
                bail!("arity mismatch for {name}");
            }
            self.vars_stack.push(HashMap::new());
            for (p, v) in params.iter().zip(values.iter()) {
                self.vars_mut().insert(p.clone(), v.clone());
            }
            let gv = self.eval_expr(guard_expr.clone())?;
            let pass = matches!(gv, Value::Bool(true));
            if pass {
                let ret = self.exec_block(&body)?;
                self.vars_stack.pop();
                return Ok(ret.unwrap_or(Value::Int(0)));
            }
            self.vars_stack.pop();
        }
        bail!("no guard matched for {name}")
    }

    // Very shallow exhaustiveness check: if scrutinee is a constructor value (or Ident referencing a constructor) and match arms enumerate some constructors without wildcard,
    // warn about missing constructors of the same type. Since types are not tracked, we approximate by collecting constructor names used in patterns and comparing against
    // all constructors known globally whose arity matches the arities seen. This is heuristic but useful for early feedback.
    fn check_match_exhaustiveness(
        &self,
        _scrutinee_expr: &Expr,
        arms: &[(Pattern, Option<Expr>, Expr)],
    ) -> Result<()> {
        // If any wildcard present, treat as exhaustive
        if arms.iter().any(|(p, _, _)| matches!(p, Pattern::Wildcard)) {
            return Ok(());
        }
        // Collect constructor names used in top-level patterns
        let mut used: Vec<String> = Vec::new();
        for (p, _, _) in arms {
            if let Pattern::Constructor { name, .. } = p {
                if !used.contains(name) {
                    used.push(name.clone());
                }
            }
        }
        if used.is_empty() {
            return Ok(());
        }
        // Determine candidate type: pick first used constructor and map to its type (reverse map).
        if let Some(first_ctor) = used.first() {
            if let Some(ty) = self.ctor_type.get(first_ctor) {
                if let Some(ctors) = self.type_ctors.get(ty) {
                    let missing: Vec<&String> =
                        ctors.iter().filter(|c| !used.contains(c)).collect();
                    if !missing.is_empty() && std::env::var("TONG_NO_MATCH_WARN").is_err() {
                        let mut first = true;
                        let mut list = String::new();
                        for m in missing {
                            if !first {
                                list.push(',');
                            }
                            list.push_str(m);
                            first = false;
                        }
                        eprintln!("[TONG][warn] non-exhaustive match for type '{ty}'; missing constructors: {list}");
                    }
                }
            }
        }
        Ok(())
    }

    fn check_match_redundancy(&self, arms: &[(Pattern, Option<Expr>, Expr)]) {
        // Simple, heuristic redundancy detection.
        use Pattern::*;
        fn key(p: &Pattern) -> Option<String> {
            match p {
                Wildcard => Some("_".into()),
                Int(i) => Some(format!("Int:{i}")),
                Bool(b) => Some(format!("Bool:{b}")),
                Constructor { name, arity, .. } => Some(format!("Ctor:{name}:{arity}")),
                Tuple(ts) => Some(format!("Tuple:{}", ts.len())),
                Ident(_) => None, // variable pattern always binds freshly; duplicates allowed
            }
        }
        let mut seen_unconditional = std::collections::HashSet::new();
        let mut wildcard_seen = false;
        for (idx, (pat, guard, _body)) in arms.iter().enumerate() {
            if wildcard_seen {
                // If earlier wildcard had a guard, later arms might still be relevant; only treat as unreachable if earlier wildcard had no guard
                if seen_unconditional.contains("_") && std::env::var("TONG_NO_MATCH_WARN").is_err()
                {
                    eprintln!("[TONG][warn] unreachable match arm #{idx} (follows wildcard)");
                }
                continue;
            }
            let is_unconditional_guard = guard.is_none();
            if let Some(k) = key(pat) {
                if k == "_" {
                    if is_unconditional_guard {
                        seen_unconditional.insert(k.clone());
                    }
                    wildcard_seen = true;
                    continue;
                }
                if is_unconditional_guard {
                    if seen_unconditional.contains(&k)
                        && std::env::var("TONG_NO_MATCH_WARN").is_err()
                    {
                        eprintln!(
                            "[TONG][warn] redundant match arm #{idx} (pattern already covered)"
                        );
                    } else {
                        seen_unconditional.insert(k);
                    }
                }
            }
        }
    }
}

fn format_value(v: &Value) -> String {
    match v {
        Value::Str(s) => s.clone(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => {
            if f.fract() == 0.0 {
                format!("{:.1}", f)
            } else {
                format!("{}", f)
            }
        }
        Value::Bool(b) => b.to_string(),
        Value::Array(items) => {
            let parts: Vec<String> = items.iter().map(format_value).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Lambda { .. } => "<lambda>".to_string(),
        Value::FuncRef(name) => format!("<func:{}>", name),
        Value::Object(_) => "<object>".to_string(),
        Value::Constructor { name, fields } => {
            if fields.is_empty() {
                name.clone()
            } else {
                format!(
                    "{}({})",
                    name,
                    fields
                        .iter()
                        .map(format_value)
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
        }
        Value::Partial { name, applied } => format!("<partial:{}:{}>", name, applied.len()),
    }
}

fn expr_from_value(v: &Value) -> Expr {
    match v {
        Value::Str(s) => Expr::Str(s.clone()),
        Value::Int(i) => Expr::Int(*i),
        Value::Float(f) => Expr::Float(*f),
        Value::Bool(b) => Expr::Bool(*b),
        Value::Array(items) => Expr::Array(items.iter().map(expr_from_value).collect()),
        Value::Lambda { params, body, .. } => Expr::Lambda {
            params: params.clone(),
            body: body.clone(),
        },
        Value::FuncRef(name) => Expr::Ident(name.clone()),
        Value::Object(_) => Expr::Ident("<object>".to_string()),
        Value::Constructor { name, .. } => Expr::Ident(name.clone()),
        Value::Partial { name, .. } => Expr::Ident(name.clone()),
    }
}

// For builtin dispatch where we already have evaluated values, keep non-expressible values as-is via a placeholder approach.
// no-op

impl Env {
    fn match_pattern(&mut self, pat: &Pattern, v: &Value) -> Result<bool> {
        Ok(match pat {
            Pattern::Wildcard => true,
            Pattern::Ident(name) => {
                self.vars_mut().insert(name.clone(), v.clone());
                true
            }
            Pattern::Int(i) => matches!(v, Value::Int(j) if j==i),
            Pattern::Bool(b) => matches!(v, Value::Bool(bb) if bb==b),
            Pattern::Constructor { name, arity, sub } => {
                if let Value::Constructor { name: cn, fields } = v {
                    if cn == name && fields.len() == *arity {
                        for (sp, fv) in sub.iter().zip(fields.iter()) {
                            if !self.match_pattern(sp, fv)? {
                                return Ok(false);
                            }
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Pattern::Tuple(subs) => {
                if let Value::Array(items) = v {
                    if items.len() != subs.len() {
                        return Ok(false);
                    }
                    for (p, iv) in subs.iter().zip(items.iter()) {
                        if !self.match_pattern(p, iv)? {
                            return Ok(false);
                        }
                    }
                    true
                } else {
                    false
                }
            }
        })
    }
    fn import_module(&mut self, name: &str) -> Result<Value> {
        if let Some(v) = self.modules.get(name) {
            return Ok(v.clone());
        }
        match name {
            "sdl" => {
                let v = self.import_sdl();
                self.modules.insert(name.to_string(), v.clone());
                Ok(v)
            }
            "linalg" => {
                let v = self.import_linalg();
                self.modules.insert(name.to_string(), v.clone());
                Ok(v)
            }
            other => bail!("unknown module '{}'; built-ins: sdl, linalg", other),
        }
    }

    fn import_sdl(&mut self) -> Value {
        let mut obj = HashMap::new();
        #[cfg(not(feature = "sdl3"))]
        {
            // Provide a one-time notice that this build is headless for SDL.
            if !self.modules.contains_key("__sdl_notice_shown") {
                eprintln!("[TONG][SDL] Built without 'sdl3' feature: using headless shim (no real window). Rebuild with --features sdl3 for graphics.");
                self.modules
                    .insert("__sdl_notice_shown".to_string(), Value::Bool(true));
            }
        }
        // constants
        obj.insert("K_ESCAPE".to_string(), Value::Int(27));
        obj.insert("K_Q".to_string(), Value::Int(81));
        obj.insert("K_W".to_string(), Value::Int(87));
        obj.insert("K_S".to_string(), Value::Int(83));
        obj.insert("K_UP".to_string(), Value::Int(1000));
        obj.insert("K_DOWN".to_string(), Value::Int(1001));
        // functions (method names map to builtin function identifiers)
        obj.insert("init".into(), Value::FuncRef("sdl_init".into()));
        obj.insert(
            "create_window".into(),
            Value::FuncRef("sdl_create_window".into()),
        );
        obj.insert(
            "create_renderer".into(),
            Value::FuncRef("sdl_create_renderer".into()),
        );
        obj.insert(
            "set_draw_color".into(),
            Value::FuncRef("sdl_set_draw_color".into()),
        );
        obj.insert("clear".into(), Value::FuncRef("sdl_clear".into()));
        obj.insert("fill_rect".into(), Value::FuncRef("sdl_fill_rect".into()));
        obj.insert("present".into(), Value::FuncRef("sdl_present".into()));
        obj.insert("delay".into(), Value::FuncRef("sdl_delay".into()));
        obj.insert("poll_quit".into(), Value::FuncRef("sdl_poll_quit".into()));
        obj.insert("key_down".into(), Value::FuncRef("sdl_key_down".into()));
        obj.insert(
            "destroy_renderer".into(),
            Value::FuncRef("sdl_destroy_renderer".into()),
        );
        obj.insert(
            "destroy_window".into(),
            Value::FuncRef("sdl_destroy_window".into()),
        );
        obj.insert("quit".into(), Value::FuncRef("sdl_quit".into()));
        Value::Object(obj)
    }

    fn call_sdl_builtin(&mut self, name: &str, args: Vec<Expr>) -> Result<Value> {
        #[cfg(feature = "sdl3")]
        {
            self.call_sdl_builtin_real(name, args)
        }
        #[cfg(not(feature = "sdl3"))]
        {
            match name {
                "sdl_init" => Ok(Value::Int(0)),
                "sdl_create_window" => Ok(Value::Int(1)),
                "sdl_create_renderer" => Ok(Value::Int(1)),
                "sdl_set_draw_color" => Ok(Value::Int(0)),
                "sdl_clear" => Ok(Value::Int(0)),
                "sdl_fill_rect" => Ok(Value::Int(0)),
                "sdl_present" => Ok(Value::Int(0)),
                "sdl_delay" => {
                    // Simulate ~60 FPS by increasing frame count; no sleeping for CI speed
                    let _ = args; // ignore actual ms
                    self.sdl_frame += 1;
                    Ok(Value::Int(0))
                }
                "sdl_poll_quit" => {
                    let quit = self.sdl_frame >= 300; // auto-quit after ~300 frames
                    Ok(Value::Bool(quit))
                }
                "sdl_key_down" => Ok(Value::Bool(false)),
                "sdl_destroy_renderer" => Ok(Value::Int(0)),
                "sdl_destroy_window" => Ok(Value::Int(0)),
                "sdl_quit" => Ok(Value::Int(0)),
                other => bail!("unknown SDL builtin {}", other),
            }
        }
    }
}

impl Env {
    fn import_linalg(&mut self) -> Value {
        let mut obj = HashMap::new();
        // function refs
        for (k, v) in [
            ("zeros", "linalg_zeros"),
            ("ones", "linalg_ones"),
            ("tensor", "linalg_tensor"),
            ("shape", "linalg_shape"),
            ("rank", "linalg_rank"),
            ("get", "linalg_get"),
            ("set", "linalg_set"),
            ("add", "linalg_add"),
            ("sub", "linalg_sub"),
            ("mul", "linalg_mul"),
            ("dot", "linalg_dot"),
            ("matmul", "linalg_matmul"),
            ("transpose", "linalg_transpose"),
        ] {
            obj.insert(k.to_string(), Value::FuncRef(v.to_string()));
        }
        Value::Object(obj)
    }

    fn call_linalg_builtin_values(&mut self, name: &str, values: Vec<Value>) -> Result<Value> {
        // helpers
        fn to_usize_vec(v: &Value) -> Result<Vec<usize>> {
            match v {
                Value::Array(items) => items
                    .iter()
                    .map(|x| match x {
                        Value::Int(i) if *i >= 0 => Ok(*i as usize),
                        _ => bail!("shape/index must be non-negative ints"),
                    })
                    .collect(),
                _ => bail!("expected array of ints"),
            }
        }
        fn to_f64_vec(v: &Value) -> Result<Vec<f64>> {
            match v {
                Value::Array(items) => items
                    .iter()
                    .map(|x| match x {
                        Value::Int(i) => Ok(*i as f64),
                        Value::Float(f) => Ok(*f),
                        _ => bail!("expected numeric array"),
                    })
                    .collect(),
                _ => bail!("expected numeric array"),
            }
        }
        fn new_tensor(data: Vec<f64>, shape: Vec<usize>) -> Value {
            let mut obj = HashMap::new();
            obj.insert("__tensor__".to_string(), Value::Bool(true));
            obj.insert(
                "shape".to_string(),
                Value::Array(shape.iter().map(|d| Value::Int(*d as i64)).collect()),
            );
            obj.insert(
                "data".to_string(),
                Value::Array(data.iter().map(|f| Value::Float(*f)).collect()),
            );
            Value::Object(obj)
        }
        fn is_tensor(v: &Value) -> Option<(Vec<usize>, Vec<f64>)> {
            if let Value::Object(map) = v {
                if let Some(Value::Bool(true)) = map.get("__tensor__") {
                    if let (Some(Value::Array(shape_vals)), Some(Value::Array(data_vals))) =
                        (map.get("shape"), map.get("data"))
                    {
                        let mut shape = Vec::new();
                        for sv in shape_vals {
                            if let Value::Int(i) = sv {
                                shape.push(*i as usize);
                            } else {
                                return None;
                            }
                        }
                        let mut data = Vec::new();
                        for dv in data_vals {
                            match dv {
                                Value::Int(i) => data.push(*i as f64),
                                Value::Float(f) => data.push(*f),
                                _ => return None,
                            }
                        }
                        return Some((shape, data));
                    }
                }
            }
            None
        }
        fn numel(shape: &[usize]) -> usize {
            shape.iter().product()
        }
        fn flat_index(shape: &[usize], idx: &[usize]) -> Result<usize> {
            if shape.len() != idx.len() {
                bail!("index rank mismatch");
            }
            let mut stride = 1usize;
            let mut strides = vec![0; shape.len()];
            for (i, d) in shape.iter().enumerate().rev() {
                strides[i] = stride;
                stride *= *d;
            }
            let mut off = 0usize;
            for (i, &ix) in idx.iter().enumerate() {
                if ix >= shape[i] {
                    bail!("index out of bounds");
                }
                off += ix * strides[i];
            }
            Ok(off)
        }

        match name {
            "linalg_zeros" => {
                if values.len() != 1 {
                    bail!("zeros(shape) expects 1 arg");
                }
                let shape = to_usize_vec(&values[0])?;
                let n = numel(&shape);
                Ok(new_tensor(vec![0.0; n], shape))
            }
            "linalg_ones" => {
                if values.len() != 1 {
                    bail!("ones(shape) expects 1 arg");
                }
                let shape = to_usize_vec(&values[0])?;
                let n = numel(&shape);
                Ok(new_tensor(vec![1.0; n], shape))
            }
            "linalg_tensor" => {
                if values.len() != 2 {
                    bail!("tensor(data, shape) expects 2 args");
                }
                let data = to_f64_vec(&values[0])?;
                let shape = to_usize_vec(&values[1])?;
                if data.len() != numel(&shape) {
                    bail!("data length does not match shape");
                }
                Ok(new_tensor(data, shape))
            }
            "linalg_shape" => {
                if values.len() != 1 {
                    bail!("shape(t) expects 1 arg");
                }
                if let Some((shape, _)) = is_tensor(&values[0]) {
                    Ok(Value::Array(
                        shape.into_iter().map(|d| Value::Int(d as i64)).collect(),
                    ))
                } else {
                    bail!("argument is not a tensor")
                }
            }
            "linalg_rank" => {
                if values.len() != 1 {
                    bail!("rank(t) expects 1 arg");
                }
                if let Some((shape, _)) = is_tensor(&values[0]) {
                    Ok(Value::Int(shape.len() as i64))
                } else {
                    bail!("argument is not a tensor")
                }
            }
            "linalg_get" => {
                if values.len() != 2 {
                    bail!("get(t, idx) expects 2 args");
                }
                if let Some((shape, data)) = is_tensor(&values[0]) {
                    let idxs = to_usize_vec(&values[1])?;
                    let fi = flat_index(&shape, &idxs)?;
                    Ok(Value::Float(data[fi]))
                } else {
                    bail!("argument is not a tensor")
                }
            }
            "linalg_set" => {
                if values.len() != 3 {
                    bail!("set(t, idx, v) expects 3 args");
                }
                if let Some((shape, mut data)) = is_tensor(&values[0]) {
                    let idxs = to_usize_vec(&values[1])?;
                    let fi = flat_index(&shape, &idxs)?;
                    let val = match &values[2] {
                        Value::Int(i) => *i as f64,
                        Value::Float(f) => *f,
                        _ => bail!("value must be numeric"),
                    };
                    data[fi] = val;
                    Ok(new_tensor(data, shape))
                } else {
                    bail!("argument is not a tensor")
                }
            }
            "linalg_add" | "linalg_sub" | "linalg_mul" => {
                if values.len() != 2 {
                    bail!("binary elementwise expects 2 args");
                }
                let (shape_a, data_a) =
                    is_tensor(&values[0]).ok_or_else(|| anyhow::anyhow!("first arg not tensor"))?;
                let (shape_b, data_b) = is_tensor(&values[1])
                    .ok_or_else(|| anyhow::anyhow!("second arg not tensor"))?;
                if shape_a != shape_b {
                    bail!("shape mismatch");
                }
                let data: Vec<f64> = data_a
                    .iter()
                    .zip(data_b.iter())
                    .map(|(a, b)| match name {
                        "linalg_add" => a + b,
                        "linalg_sub" => a - b,
                        _ => a * b,
                    })
                    .collect();
                Ok(new_tensor(data, shape_a))
            }
            "linalg_dot" => {
                if values.len() != 2 {
                    bail!("dot(a,b) expects 2 args");
                }
                let (shape_a, data_a) =
                    is_tensor(&values[0]).ok_or_else(|| anyhow::anyhow!("first arg not tensor"))?;
                let (shape_b, data_b) = is_tensor(&values[1])
                    .ok_or_else(|| anyhow::anyhow!("second arg not tensor"))?;
                if shape_a.len() != 1 || shape_b.len() != 1 {
                    bail!("dot expects 1-D tensors");
                }
                if shape_a[0] != shape_b[0] {
                    bail!("length mismatch");
                }
                let mut s = 0.0;
                for (a, b) in data_a.iter().zip(data_b.iter()) {
                    s += a * b;
                }
                Ok(Value::Float(s))
            }
            "linalg_matmul" => {
                if values.len() != 2 {
                    bail!("matmul(a,b) expects 2 args");
                }
                let (sa, da) =
                    is_tensor(&values[0]).ok_or_else(|| anyhow::anyhow!("first arg not tensor"))?;
                let (sb, db) = is_tensor(&values[1])
                    .ok_or_else(|| anyhow::anyhow!("second arg not tensor"))?;
                if sa.len() != 2 || sb.len() != 2 {
                    bail!("matmul expects 2-D tensors");
                }
                if sa[1] != sb[0] {
                    bail!("inner dimension mismatch");
                }
                let (m, k, n) = (sa[0], sa[1], sb[1]);
                let mut out = vec![0.0; m * n];
                for i in 0..m {
                    for j in 0..n {
                        let mut acc = 0.0;
                        for p in 0..k {
                            acc += da[i * k + p] * db[p * n + j];
                        }
                        out[i * n + j] = acc;
                    }
                }
                Ok(new_tensor(out, vec![m, n]))
            }
            "linalg_transpose" => {
                if values.len() != 1 {
                    bail!("transpose(a) expects 1 arg");
                }
                let (s, d) =
                    is_tensor(&values[0]).ok_or_else(|| anyhow::anyhow!("argument not tensor"))?;
                if s.len() != 2 {
                    bail!("transpose expects rank-2 tensor");
                }
                let (m, n) = (s[0], s[1]);
                let mut out = vec![0.0; m * n];
                for i in 0..m {
                    for j in 0..n {
                        out[j * m + i] = d[i * n + j];
                    }
                }
                Ok(new_tensor(out, vec![s[1], s[0]]))
            }
            _ => bail!("unknown linalg builtin {}", name),
        }
    }
}

// ---------------- REPL SUPPORT (public minimal API) -----------------
pub struct Repl {
    env: Env,
}

impl Repl {
    pub fn new() -> Self {
        Self { env: Env::new() }
    }

    // Evaluate a source snippet, returning an optional printable value (final bare expression)
    pub fn eval_snippet(&mut self, src: &str) -> Result<Option<String>> {
        // Lex & parse new snippet each time; keep accumulated functions / vars
        let tokens = crate::lexer::lex(src)?;
        let program = crate::parser::parse(tokens)?;

        // First collect function/main definitions without clearing existing ones
        for stmt in &program.stmts {
            match stmt {
                Stmt::FnDef(name, params, body) => {
                    // Plain function: overwrite previous definition
                    self.env
                        .funcs
                        .insert(name.clone(), (params.clone(), body.clone()));
                }
                Stmt::FnDefGuarded(name, params, guard, body) => {
                    // Append guarded clause to existing set (REPL allows incremental clause authoring)
                    self.env
                        .guarded_funcs
                        .entry(name.clone())
                        .or_default()
                        .push((params.clone(), guard.clone(), body.clone()));
                }
                Stmt::FnDefPattern(name, patterns, guard, body) => {
                    // Append pattern clause maintaining order of entry across snippets
                    self.env
                        .pattern_funcs
                        .entry(name.clone())
                        .or_default()
                        .push((patterns.clone(), guard.clone(), body.clone()));
                }
                Stmt::FnMain(body) => {
                    self.env
                        .funcs
                        .insert("main".to_string(), (Vec::new(), body.clone()));
                }
                Stmt::DataDecl(_tname, ctors) => {
                    for c in ctors {
                        self.env.data_ctors.insert(c.name.clone(), c.arity);
                    }
                }
                _ => {}
            }
        }

        // Execute non-function statements; remember last expression value if it was a bare Expr stmt
        let mut last_expr: Option<Value> = None;
        for stmt in &program.stmts {
            match stmt {
                Stmt::Import(name, module) => {
                    let v = self.env.import_module(module)?;
                    self.env.vars_mut().insert(name.clone(), v);
                }
                Stmt::Let(name, expr) => {
                    let v = self.env.eval_expr(expr.clone())?;
                    self.env.vars_mut().insert(name.clone(), v);
                }
                Stmt::Assign(name, expr) => {
                    let v = self.env.eval_expr(expr.clone())?;
                    self.env.vars_mut().insert(name.clone(), v);
                }
                Stmt::Print(args) => {
                    let parts: Result<Vec<String>> = args
                        .iter()
                        .cloned()
                        .map(|e| self.env.eval_expr(e).map(|v| format_value(&v)))
                        .collect();
                    println!("{}", parts?.join(" "));
                    last_expr = None; // print supersedes expression echo
                }
                Stmt::Expr(e) => {
                    let v = self.env.eval_expr(e.clone())?;
                    last_expr = Some(v);
                }
                _ => {
                    // control flow / while / if at top-level are executed via exec_stmt path
                    // For simplicity reuse exec_stmt for those
                    match stmt {
                        Stmt::If(..) | Stmt::While(..) | Stmt::Parallel(..) | Stmt::Return(..) => {
                            let _ = self.env.exec_stmt(stmt)?;
                            last_expr = None;
                        }
                        Stmt::FnDef(..)
                        | Stmt::FnDefGuarded(..)
                        | Stmt::FnDefPattern(..)
                        | Stmt::FnMain(..)
                        | Stmt::DataDecl(..) => {}
                        _ => {}
                    }
                }
            }
        }
        Ok(last_expr.map(|v| format_value(&v)))
    }

    pub fn list_vars(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for (k, v) in self.env.vars() {
            if k.starts_with("__") {
                continue;
            }
            out.push((k.clone(), format_value(v)));
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    pub fn reset(&mut self) {
        self.env = Env::new();
    }
}

#[cfg(feature = "sdl3")]
struct SdlState {
    _sdl: sdl3::Sdl,
    video: sdl3::VideoSubsystem,
    window: Option<sdl3::video::Window>,
    canvas: Option<sdl3::render::Canvas<sdl3::video::Window>>,
    events: sdl3::EventPump,
    draw_color: (u8, u8, u8, u8),
}

#[cfg(feature = "sdl3")]
impl Env {
    fn sdl_state_mut(&mut self) -> Result<&mut SdlState> {
        if self.sdl.is_none() {
            let sdl = sdl3::init().map_err(|e| anyhow!(e))?;
            let video = sdl.video().map_err(|e| anyhow!(e))?;
            let events = sdl.event_pump().map_err(|e| anyhow!(e))?;
            self.sdl = Some(SdlState {
                _sdl: sdl,
                video,
                window: None,
                canvas: None,
                events,
                draw_color: (0, 0, 0, 255),
            });
        }
        Ok(self.sdl.as_mut().unwrap())
    }

    fn call_sdl_builtin_real(&mut self, name: &str, args: Vec<Expr>) -> Result<Value> {
        use sdl3::{event::Event, keyboard::Scancode, pixels::Color, rect::Rect};
        match name {
            "sdl_init" => {
                let _ = self.sdl_state_mut()?; // ensure initialized
                Ok(Value::Int(0))
            }
            "sdl_create_window" => {
                // evaluate arguments first to avoid borrow conflicts
                let title = match args.first().map(|e| self.eval_expr(e.clone())) {
                    Some(Ok(Value::Str(s))) => s,
                    _ => "TONG".to_string(),
                };
                let w = match args.get(1).map(|e| self.eval_expr(e.clone())) {
                    Some(Ok(Value::Int(i))) => i as u32,
                    _ => 800,
                };
                let h = match args.get(2).map(|e| self.eval_expr(e.clone())) {
                    Some(Ok(Value::Int(i))) => i as u32,
                    _ => 600,
                };
                let state = self.sdl_state_mut()?;
                let window = state
                    .video
                    .window(&title, w, h)
                    .position_centered()
                    .build()
                    .map_err(|e| anyhow!(e))?;
                state.window = Some(window);
                Ok(Value::Int(1))
            }
            "sdl_create_renderer" => {
                let state = self.sdl_state_mut()?;
                let window = state
                    .window
                    .take()
                    .ok_or_else(|| anyhow!("create_renderer: window not created"))?;
                // sdl3 API: into_canvas() returns a Canvas directly (no builder chain)
                let canvas = window.into_canvas();
                state.canvas = Some(canvas);
                Ok(Value::Int(1))
            }
            "sdl_set_draw_color" => {
                let (r, g, b, a) = (
                    self.eval_expr(args[1].clone())?.as_int_u8()?,
                    self.eval_expr(args[2].clone())?.as_int_u8()?,
                    self.eval_expr(args[3].clone())?.as_int_u8()?,
                    self.eval_expr(args[4].clone())?.as_int_u8()?,
                );
                let state = self.sdl_state_mut()?;
                let canvas = state
                    .canvas
                    .as_mut()
                    .ok_or_else(|| anyhow!("renderer not created"))?;
                canvas.set_draw_color(Color::RGBA(r, g, b, a));
                state.draw_color = (r, g, b, a);
                Ok(Value::Int(0))
            }
            "sdl_clear" => {
                let state = self.sdl_state_mut()?;
                let canvas = state
                    .canvas
                    .as_mut()
                    .ok_or_else(|| anyhow!("renderer not created"))?;
                canvas.clear();
                Ok(Value::Int(0))
            }
            "sdl_fill_rect" => {
                // args: (ren, x,y,w,h, r,g,b,a)
                let x = self.eval_expr(args[1].clone())?.as_int_i32()?;
                let y = self.eval_expr(args[2].clone())?.as_int_i32()?;
                let w = self.eval_expr(args[3].clone())?.as_int_u32()?;
                let h = self.eval_expr(args[4].clone())?.as_int_u32()?;
                let (r, g, b, a) = (
                    self.eval_expr(args[5].clone())?.as_int_u8()?,
                    self.eval_expr(args[6].clone())?.as_int_u8()?,
                    self.eval_expr(args[7].clone())?.as_int_u8()?,
                    self.eval_expr(args[8].clone())?.as_int_u8()?,
                );
                let state = self.sdl_state_mut()?;
                let canvas = state
                    .canvas
                    .as_mut()
                    .ok_or_else(|| anyhow!("renderer not created"))?;
                let prev = state.draw_color;
                canvas.set_draw_color(Color::RGBA(r, g, b, a));
                canvas.fill_rect(Rect::new(x, y, w, h)).ok();
                canvas.set_draw_color(Color::RGBA(prev.0, prev.1, prev.2, prev.3));
                Ok(Value::Int(0))
            }
            "sdl_present" => {
                let state = self.sdl_state_mut()?;
                let canvas = state
                    .canvas
                    .as_mut()
                    .ok_or_else(|| anyhow!("renderer not created"))?;
                canvas.present();
                Ok(Value::Int(0))
            }
            "sdl_delay" => {
                let ms = match args.first().map(|e| self.eval_expr(e.clone())) {
                    Some(Ok(Value::Int(i))) => i,
                    _ => 16,
                };
                std::thread::sleep(Duration::from_millis(ms as u64));
                Ok(Value::Int(0))
            }
            "sdl_poll_quit" => {
                let state = self.sdl_state_mut()?;
                let mut quit = false;
                for event in state.events.poll_iter() {
                    if let Event::Quit { .. } = event {
                        quit = true;
                        break;
                    }
                }
                Ok(Value::Bool(quit))
            }
            "sdl_key_down" => {
                let code = match args.first().map(|e| self.eval_expr(e.clone())) {
                    Some(Ok(Value::Int(i))) => i,
                    _ => 0,
                };
                let state = self.sdl_state_mut()?;
                let kb = state.events.keyboard_state();
                let pressed = match code {
                    27 => kb.is_scancode_pressed(Scancode::Escape),
                    81 => kb.is_scancode_pressed(Scancode::Q),
                    87 => kb.is_scancode_pressed(Scancode::W),
                    83 => kb.is_scancode_pressed(Scancode::S),
                    1000 => kb.is_scancode_pressed(Scancode::Up),
                    1001 => kb.is_scancode_pressed(Scancode::Down),
                    _ => false,
                };
                Ok(Value::Bool(pressed))
            }
            "sdl_destroy_renderer" => {
                let state = self.sdl_state_mut()?;
                let _ = args; // ignore handle
                state.canvas = None;
                Ok(Value::Int(0))
            }
            "sdl_destroy_window" => {
                let state = self.sdl_state_mut()?;
                let _ = args; // ignore handle
                state.window = None;
                Ok(Value::Int(0))
            }
            "sdl_quit" => {
                // Drop everything
                if let Some(st) = self.sdl.as_mut() {
                    st.window = None;
                }
                Ok(Value::Int(0))
            }
            other => bail!("unknown SDL builtin {}", other),
        }
    }
}

// Helper conversions for Value to numeric types (only needed when SDL backend active)
#[cfg(feature = "sdl3")]
impl Value {
    fn as_int_u8(&self) -> Result<u8> {
        match self {
            Value::Int(i) => Ok((*i).clamp(0, 255) as u8),
            _ => bail!("expected int"),
        }
    }
    fn as_int_u32(&self) -> Result<u32> {
        match self {
            Value::Int(i) => Ok((*i).max(0) as u32),
            _ => bail!("expected int"),
        }
    }
    fn as_int_i32(&self) -> Result<i32> {
        match self {
            Value::Int(i) => Ok(*i as i32),
            _ => bail!("expected int"),
        }
    }
}
