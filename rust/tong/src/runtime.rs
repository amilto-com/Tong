use anyhow::{anyhow, bail, Result};
use std::collections::HashMap;
use std::time::Duration;

use crate::parser::{BinOp, Expr, Program, Stmt};

pub fn execute(program: Program) -> Result<()> {
    let mut env = Env::new();

    // First collect function definitions
    for stmt in &program.stmts {
        if let Stmt::FnDef(name, params, body) = stmt { env.funcs.insert(name.clone(), (params.clone(), body.clone())); }
        if let Stmt::FnMain(body) = stmt { env.funcs.insert("main".to_string(), (Vec::new(), body.clone())); }
    }

    // Execute top-level statements (import/let/assign/print)
    for stmt in &program.stmts {
        match stmt {
            Stmt::Import(name, module) => { let v = env.import_module(module)?; env.vars_mut().insert(name.clone(), v); }
            Stmt::Let(name, expr) => { let v = env.eval_expr(expr.clone())?; env.vars_mut().insert(name.clone(), v); }
            Stmt::Assign(name, expr) => { let v = env.eval_expr(expr.clone())?; env.vars_mut().insert(name.clone(), v); }
            Stmt::Print(args) => { let parts: Result<Vec<String>> = args.iter().cloned().map(|e| env.eval_expr(e).map(|v| format_value(&v))).collect(); println!("{}", parts?.join(" ")); }
            Stmt::Expr(e) => { let _ = env.eval_expr(e.clone())?; }
            _ => {}
        }
    }

    // Call main() if present unless the script calls it explicitly; to avoid double-run, we no longer auto-invoke here.

    Ok(())
}

#[derive(Debug, Clone)]
enum Value {
    Str(String),
    Float(f64),
    Int(i64),
    Bool(bool),
    Array(Vec<Value>),
    Lambda { params: Vec<String>, body: Box<Expr>, env: HashMap<String, Value> },
    FuncRef(String),
    Object(HashMap<String, Value>),
}

#[derive(Default)]
struct Env {
    vars_stack: Vec<HashMap<String, Value>>, // lexical-style stack
    funcs: HashMap<String, (Vec<String>, Vec<Stmt>)>,
    modules: HashMap<String, Value>, // loaded modules
    sdl_frame: i64, // headless SDL shim frame counter
    #[cfg(feature = "sdl3")]
    sdl: Option<SdlState>,
}

impl Env {
    // Execute a sequence of statements. Return Some(value) if a Return was hit.
    fn exec_block(&mut self, block: &[Stmt]) -> Result<Option<Value>> {
        for s in block {
            if let Some(rv) = self.exec_stmt(s)? { return Ok(Some(rv)); }
        }
        Ok(None)
    }

    // Execute a single statement. Return Some(value) if a Return was hit.
    fn exec_stmt(&mut self, s: &Stmt) -> Result<Option<Value>> {
        match s {
            Stmt::Import(n, m) => { let v = self.import_module(m)?; self.vars_mut().insert(n.clone(), v); Ok(None) }
            Stmt::Let(n, e) => { let v = self.eval_expr(e.clone())?; self.vars_mut().insert(n.clone(), v); Ok(None) }
            Stmt::Assign(n, e) => { let v = self.eval_expr(e.clone())?; self.vars_mut().insert(n.clone(), v); Ok(None) }
            Stmt::Print(args) => { let mut parts = Vec::new(); for e in args { parts.push(format_value(&self.eval_expr(e.clone())?)); } println!("{}", parts.join(" ")); Ok(None) }
            Stmt::Return(e) => { let v = self.eval_expr(e.clone())?; Ok(Some(v)) }
            Stmt::FnDef(name, params, body) => { self.funcs.insert(name.clone(), (params.clone(), body.clone())); Ok(None) }
            Stmt::Expr(e) => { let _ = self.eval_expr(e.clone())?; Ok(None) }
            Stmt::FnMain(_) => Ok(None),
            Stmt::If(cond, then_body, else_body) => {
                let v = self.eval_expr(cond.clone())?;
                if matches!(v, Value::Bool(true)) {
                    self.exec_block(then_body)
                } else if let Some(eb) = else_body { self.exec_block(eb) } else { Ok(None) }
            }
            Stmt::While(cond, body) => {
                loop {
                    let v = self.eval_expr(cond.clone())?;
                    if !matches!(v, Value::Bool(true)) { break; }
                    if let Some(rv) = self.exec_block(body)? { return Ok(Some(rv)); }
                }
                Ok(None)
            }
            Stmt::Parallel(inner) => {
                for is in inner { if let Some(rv) = self.exec_stmt(is)? { return Ok(Some(rv)); } }
                Ok(None)
            }
        }
    }
    fn new() -> Self { Self { vars_stack: vec![HashMap::new()], funcs: HashMap::new(), modules: HashMap::new(), sdl_frame: 0, #[cfg(feature="sdl3")] sdl: None } }
    fn vars(&self) -> &HashMap<String, Value> { self.vars_stack.last().unwrap() }
    fn vars_mut(&mut self) -> &mut HashMap<String, Value> { self.vars_stack.last_mut().unwrap() }
    fn get_var(&self, name: &str) -> Option<Value> {
        for frame in self.vars_stack.iter().rev() {
            if let Some(v) = frame.get(name) { return Some(v.clone()); }
        }
        None
    }

    fn eval_expr(&mut self, e: Expr) -> Result<Value> {
        Ok(match e {
            Expr::Property { target, name } => {
                let obj = self.eval_expr(*target)?;
                match obj {
                    Value::Object(map) => map.get(&name).cloned().ok_or_else(|| anyhow::anyhow!(format!("unknown property {}", name)))?,
                    _ => bail!("property access on non-object"),
                }
            }
            Expr::MethodCall{ target, method, args } => {
                let obj_val = self.eval_expr(*target)?;
                match obj_val.clone() {
                    Value::Object(map) => {
                        let func = map.get(&method).cloned().ok_or_else(|| anyhow::anyhow!("unknown method {}", method))?;
                        let mut values: Vec<Value> = Vec::new();
                        // For our module methods, we pass evaluated args; some APIs expect (handle, ...), but our API encodes handle as first arg explicitly in the .tong code.
                        for a in args { values.push(self.eval_expr(a)?); }
                        match func {
                            Value::FuncRef(name) => {
                                if name.starts_with("sdl_") {
                                    // Build Expr args: if the first arg is a renderer/window handle placeholder, map to a simple int; otherwise, just map basic values.
                                    let expr_args: Vec<Expr> = values.into_iter().map(|v| expr_from_value(&v)).collect();
                                    self.call_sdl_builtin(&name, expr_args)?
                                } else {
                                    self.call_function_values(name, values)?
                                }
                            }
                            Value::Lambda { params, body, env } => self.call_lambda_values(params, *body, env, values)?,
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
            Expr::UnaryNeg(inner) => match self.eval_expr(*inner)? { Value::Int(i) => Value::Int(-i), Value::Float(f) => Value::Float(-f), _ => bail!("unary '-' expects numeric") },
            Expr::Ident(name) => {
                if let Some(v) = self.get_var(&name) { v } else if self.funcs.contains_key(&name) { Value::FuncRef(name) } else { bail!("undefined variable {}", name) }
            }
            Expr::Array(items) => {
                let mut out = Vec::new();
                for it in items { out.push(self.eval_expr(it)?); }
                Value::Array(out)
            }
            Expr::Index(target, idx) => {
                let arr = self.eval_expr(*target)?;
                let i = self.eval_expr(*idx)?;
                match (arr, i) {
                    (Value::Array(v), Value::Int(n)) => {
                        if n < 0 { bail!("negative index not supported") };
                        let ni = n as usize;
                        v.get(ni).cloned().ok_or_else(|| anyhow::anyhow!("index out of bounds"))?
                    }
                    _ => bail!("indexing expects array[index]")
                }
            }
            Expr::Call { callee, args } => {
                match callee.as_str() {
                    "len" => {
                        if args.len() != 1 { bail!("len expects 1 argument"); }
                        let v0 = self.eval_expr(args[0].clone())?;
                        match v0 { Value::Array(v) => Value::Int(v.len() as i64), _ => bail!("len expects array") }
                    }
                    "import" => {
                        if args.len() != 1 { bail!("import expects 1 argument"); }
                        let v0 = self.eval_expr(args[0].clone())?;
                        match v0 { Value::Str(name) => self.import_module(&name)?, _ => bail!("import expects string module name") }
                    }
                    "sum" => {
                        if args.len() != 1 { bail!("sum expects 1 argument"); }
                        let v0 = self.eval_expr(args[0].clone())?;
                        match v0 {
                            Value::Array(v) => {
                                let mut is_float = false;
                                let mut total_f = 0.0f64;
                                let mut total_i: i64 = 0;
                                for it in v {
                                    match it {
                                        Value::Int(i) => total_i += i,
                                        Value::Float(f) => { total_f += f; is_float = true; }
                                        _ => bail!("sum expects numeric array"),
                                    }
                                }
                                if is_float { Value::Float(total_i as f64 + total_f) } else { Value::Int(total_i) }
                            }
                            _ => bail!("sum expects array"),
                        }
                    }
                    "filter" => {
                        if args.len() != 2 { bail!("filter expects 2 arguments (array, function)"); }
                        let arr_val = self.eval_expr(args[0].clone())?;
                        let callable = args[1].clone();
                        match arr_val {
                            Value::Array(items) => {
                                let mut out = Vec::new();
                                for item in items {
                                    let res = self.apply_callable(callable.clone(), vec![expr_from_value(&item)])?;
                                    match res {
                                        Value::Bool(true) => out.push(item),
                                        Value::Bool(false) => {},
                                        _ => bail!("filter function must return bool"),
                                    }
                                }
                                Value::Array(out)
                            }
                            _ => bail!("filter expects array as first argument"),
                        }
                    }
                    "reduce" => {
                        if args.len() != 3 { bail!("reduce expects 3 arguments (array, function, initial)"); }
                        let arr_val = self.eval_expr(args[0].clone())?;
                        let callable = args[1].clone();
                        let mut acc = self.eval_expr(args[2].clone())?;
                        match arr_val {
                            Value::Array(items) => {
                                for item in items {
                                    acc = self.apply_callable(callable.clone(), vec![expr_from_value(&acc), expr_from_value(&item)])?;
                                }
                                acc
                            }
                            _ => bail!("reduce expects array as first argument"),
                        }
                    }
                    "map" => {
                        if args.len() != 2 { bail!("map expects 2 arguments (array, function)"); }
                        let arr_val = self.eval_expr(args[0].clone())?;
                        let callable = args[1].clone();
                        match arr_val {
                            Value::Array(items) => {
                                let mut out = Vec::new();
                                for item in items {
                                    let arg_expr = expr_from_value(&item);
                                    let v = self.apply_callable(callable.clone(), vec![arg_expr])?;
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
                                    let evaled: Result<Vec<Value>> = args.into_iter().map(|a| self.eval_expr(a)).collect();
                                    self.call_lambda_values(params, *body, env, evaled?)?
                                }
                                Value::FuncRef(name) => {
                                    let evaled: Result<Vec<Value>> = args.into_iter().map(|a| self.eval_expr(a)).collect();
                                    self.call_function_values(name, evaled?)?
                                }
                                _ => bail!("{} is not callable", callee),
                            }
                        } else if self.funcs.contains_key(&callee) {
                            let evaled: Result<Vec<Value>> = args.into_iter().map(|a| self.eval_expr(a)).collect();
                            self.call_function_values(callee.clone(), evaled?)?
                        } else {
                            bail!("unknown function {}", callee)
                        }
                    }
                }
            }
            Expr::Binary { op, left, right } => {
                let l = self.eval_expr(*left)?;
                let r = self.eval_expr(*right)?;
                match (l, r, op) {
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

                    (Value::Float(a), Value::Float(b), BinOp::Eq) => Value::Bool((a - b).abs() < f64::EPSILON),
                    (Value::Int(a), Value::Int(b), BinOp::Eq) => Value::Bool(a == b),
                    (Value::Bool(a), Value::Bool(b), BinOp::Eq) => Value::Bool(a == b),
                    (Value::Str(a), Value::Str(b), BinOp::Eq) => Value::Bool(a == b),

                    (Value::Float(a), Value::Float(b), BinOp::Ne) => Value::Bool((a - b).abs() >= f64::EPSILON),
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
        })
    }

    fn apply_callable(&mut self, func: Expr, args: Vec<Expr>) -> Result<Value> {
        match func {
            Expr::Ident(name) => {
                // could be function name or variable holding function/lambda
                if let Some(v) = self.get_var(&name) {
                    match v {
                        Value::Lambda { params, body, env } => {
                            // Evaluate args then call with values to preserve object args
                            let evaled: Result<Vec<Value>> = args.into_iter().map(|a| self.eval_expr(a)).collect();
                            self.call_lambda_values(params, *body, env, evaled?)
                        }
                        Value::FuncRef(fname) => {
                            let evaled: Result<Vec<Value>> = args.into_iter().map(|a| self.eval_expr(a)).collect();
                            self.call_function_values(fname, evaled?)
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

    fn call_lambda(&mut self, params: Vec<String>, body: Expr, captured_env: HashMap<String, Value>, args: Vec<Expr>) -> Result<Value> {
        if params.len() != args.len() { bail!("arity mismatch for lambda"); }
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
    fn call_lambda_values(&mut self, params: Vec<String>, body: Expr, captured_env: HashMap<String, Value>, values: Vec<Value>) -> Result<Value> {
        if params.len() != values.len() { bail!("arity mismatch for lambda"); }
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
        // Built-in shims for SDL module
        if name.starts_with("sdl_") {
            return self.call_sdl_builtin(&name, args);
        }
        let (params, body) = self
            .funcs
            .get(&name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("unknown function {}", name))?;
        if params.len() != args.len() { bail!("arity mismatch for {}", name); }
        // Push a new frame
        self.vars_stack.push(HashMap::new());
        for (p, a) in params.iter().zip(args.into_iter()) {
            let val = self.eval_expr(a)?;
            self.vars_mut().insert(p.clone(), val);
        }

        // Execute body with nested control-flow properly handled
        let ret = self.exec_block(&body)?;
        // Pop the frame
        self.vars_stack.pop();
        Ok(ret.unwrap_or(Value::Int(0)))
    }

    // Function call with pre-evaluated values (avoids expr_from_value conversion issues for objects)
    fn call_function_values(&mut self, name: String, values: Vec<Value>) -> Result<Value> {
        if name.starts_with("sdl_") { bail!("internal: call_function_values should not be used for SDL builtins"); }
        let (params, body) = self
            .funcs
            .get(&name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("unknown function {}", name))?;
        if params.len() != values.len() { bail!("arity mismatch for {}", name); }
        self.vars_stack.push(HashMap::new());
        for (p, v) in params.iter().zip(values.into_iter()) {
            self.vars_mut().insert(p.clone(), v);
        }
        let ret = self.exec_block(&body)?;
        self.vars_stack.pop();
        Ok(ret.unwrap_or(Value::Int(0)))
    }
}

fn format_value(v: &Value) -> String {
    match v {
        Value::Str(s) => s.clone(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => {
            if f.fract() == 0.0 { format!("{:.1}", f) } else { format!("{}", f) }
        }
        Value::Bool(b) => b.to_string(),
        Value::Array(items) => {
            let parts: Vec<String> = items.iter().map(format_value).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Lambda { .. } => "<lambda>".to_string(),
        Value::FuncRef(name) => format!("<func:{}>", name),
        Value::Object(_) => "<object>".to_string(),
    }
}

fn expr_from_value(v: &Value) -> Expr {
    match v {
        Value::Str(s) => Expr::Str(s.clone()),
        Value::Int(i) => Expr::Int(*i),
        Value::Float(f) => Expr::Float(*f),
        Value::Bool(b) => Expr::Bool(*b),
        Value::Array(items) => Expr::Array(items.iter().map(expr_from_value).collect()),
        Value::Lambda { params, body, .. } => Expr::Lambda { params: params.clone(), body: body.clone() },
        Value::FuncRef(name) => Expr::Ident(name.clone()),
        Value::Object(_) => Expr::Ident("<object>".to_string()),
    }
}

// For builtin dispatch where we already have evaluated values, keep non-expressible values as-is via a placeholder approach.
fn expr_from_value_passthrough(v: Value) -> Expr {
    match v {
        Value::Str(s) => Expr::Str(s),
        Value::Int(i) => Expr::Int(i),
        Value::Float(f) => Expr::Float(f),
        Value::Bool(b) => Expr::Bool(b),
        Value::Array(items) => Expr::Array(items.iter().map(expr_from_value).collect()),
        Value::Lambda { params, body, .. } => Expr::Lambda { params, body },
        Value::FuncRef(name) => Expr::Ident(name),
        Value::Object(map) => {
            // store the object in a temp variable on the top frame and reference it; generate a unique name
            let temp_name = format!("__obj_{}", map.len());
            // Note: This helper can't access self; we'll not use it for non-SDL builtins to avoid this complexity.
            Expr::Ident(temp_name)
        }
    }
}

impl Env {
    fn import_module(&mut self, name: &str) -> Result<Value> {
        if let Some(v) = self.modules.get(name) { return Ok(v.clone()); }
        match name {
            "sdl" => {
                let v = self.import_sdl();
                self.modules.insert(name.to_string(), v.clone());
                Ok(v)
            }
            other => bail!("unknown module '{}'; built-ins: sdl", other),
        }
    }

    fn import_sdl(&mut self) -> Value {
        let mut obj = HashMap::new();
        // constants
        obj.insert("K_ESCAPE".to_string(), Value::Int(27));
        obj.insert("K_Q".to_string(), Value::Int(81));
        obj.insert("K_W".to_string(), Value::Int(87));
        obj.insert("K_S".to_string(), Value::Int(83));
        obj.insert("K_UP".to_string(), Value::Int(1000));
        obj.insert("K_DOWN".to_string(), Value::Int(1001));
        // functions (method names map to builtin function identifiers)
        obj.insert("init".into(), Value::FuncRef("sdl_init".into()));
        obj.insert("create_window".into(), Value::FuncRef("sdl_create_window".into()));
        obj.insert("create_renderer".into(), Value::FuncRef("sdl_create_renderer".into()));
        obj.insert("set_draw_color".into(), Value::FuncRef("sdl_set_draw_color".into()));
        obj.insert("clear".into(), Value::FuncRef("sdl_clear".into()));
        obj.insert("fill_rect".into(), Value::FuncRef("sdl_fill_rect".into()));
        obj.insert("present".into(), Value::FuncRef("sdl_present".into()));
        obj.insert("delay".into(), Value::FuncRef("sdl_delay".into()));
        obj.insert("poll_quit".into(), Value::FuncRef("sdl_poll_quit".into()));
        obj.insert("key_down".into(), Value::FuncRef("sdl_key_down".into()));
        obj.insert("destroy_renderer".into(), Value::FuncRef("sdl_destroy_renderer".into()));
        obj.insert("destroy_window".into(), Value::FuncRef("sdl_destroy_window".into()));
        obj.insert("quit".into(), Value::FuncRef("sdl_quit".into()));
        Value::Object(obj)
    }

    fn call_sdl_builtin(&mut self, name: &str, args: Vec<Expr>) -> Result<Value> {
        #[cfg(feature = "sdl3")]
        {
            return self.call_sdl_builtin_real(name, args);
        }
        #[cfg(not(feature = "sdl3"))]
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

#[cfg(feature = "sdl3")]
struct SdlState {
    sdl: sdl3::Sdl,
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
            self.sdl = Some(SdlState { sdl, video, window: None, canvas: None, events, draw_color: (0,0,0,255) });
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
                let title = match args.get(0).map(|e| self.eval_expr(e.clone())) { Some(Ok(Value::Str(s))) => s, _ => "TONG".to_string() };
                let w = match args.get(1).map(|e| self.eval_expr(e.clone())) { Some(Ok(Value::Int(i))) => i as u32, _ => 800 };
                let h = match args.get(2).map(|e| self.eval_expr(e.clone())) { Some(Ok(Value::Int(i))) => i as u32, _ => 600 };
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
                let window = state.window.take().ok_or_else(|| anyhow!("create_renderer: window not created"))?;
                // sdl3 API: into_canvas() returns a Canvas directly (no builder chain)
                let canvas = window.into_canvas();
                state.canvas = Some(canvas);
                Ok(Value::Int(1))
            }
            "sdl_set_draw_color" => {
                let (r,g,b,a) = (
                    self.eval_expr(args[1].clone())?.as_int_u8()?,
                    self.eval_expr(args[2].clone())?.as_int_u8()?,
                    self.eval_expr(args[3].clone())?.as_int_u8()?,
                    self.eval_expr(args[4].clone())?.as_int_u8()?,
                );
                let state = self.sdl_state_mut()?;
                let canvas = state.canvas.as_mut().ok_or_else(|| anyhow!("renderer not created"))?;
                canvas.set_draw_color(Color::RGBA(r,g,b,a));
                state.draw_color = (r,g,b,a);
                Ok(Value::Int(0))
            }
            "sdl_clear" => {
                let state = self.sdl_state_mut()?;
                let canvas = state.canvas.as_mut().ok_or_else(|| anyhow!("renderer not created"))?;
                canvas.clear();
                Ok(Value::Int(0))
            }
            "sdl_fill_rect" => {
                // args: (ren, x,y,w,h, r,g,b,a)
                let x = self.eval_expr(args[1].clone())?.as_int_i32()?;
                let y = self.eval_expr(args[2].clone())?.as_int_i32()?;
                let w = self.eval_expr(args[3].clone())?.as_int_u32()?;
                let h = self.eval_expr(args[4].clone())?.as_int_u32()?;
                let (r,g,b,a) = (
                    self.eval_expr(args[5].clone())?.as_int_u8()?,
                    self.eval_expr(args[6].clone())?.as_int_u8()?,
                    self.eval_expr(args[7].clone())?.as_int_u8()?,
                    self.eval_expr(args[8].clone())?.as_int_u8()?,
                );
                let state = self.sdl_state_mut()?;
                let canvas = state.canvas.as_mut().ok_or_else(|| anyhow!("renderer not created"))?;
                let prev = state.draw_color;
                canvas.set_draw_color(Color::RGBA(r,g,b,a));
                canvas.fill_rect(Rect::new(x, y, w, h)).ok();
                canvas.set_draw_color(Color::RGBA(prev.0, prev.1, prev.2, prev.3));
                Ok(Value::Int(0))
            }
            "sdl_present" => {
                let state = self.sdl_state_mut()?;
                let canvas = state.canvas.as_mut().ok_or_else(|| anyhow!("renderer not created"))?;
                canvas.present();
                Ok(Value::Int(0))
            }
            "sdl_delay" => {
                let ms = match args.get(0).map(|e| self.eval_expr(e.clone())) { Some(Ok(Value::Int(i))) => i, _ => 16 };
                std::thread::sleep(Duration::from_millis(ms as u64));
                Ok(Value::Int(0))
            }
            "sdl_poll_quit" => {
                let state = self.sdl_state_mut()?;
                let mut quit = false;
                for event in state.events.poll_iter() {
                    if let Event::Quit { .. } = event { quit = true; break; }
                }
                Ok(Value::Bool(quit))
            }
            "sdl_key_down" => {
                let code = match args.get(0).map(|e| self.eval_expr(e.clone())) { Some(Ok(Value::Int(i))) => i, _ => 0 };
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

// Helper conversions for Value to numeric types
impl Value {
    fn as_int_u8(&self) -> Result<u8> { match self { Value::Int(i) => Ok((*i).clamp(0, 255) as u8), _ => bail!("expected int") } }
    fn as_int_u32(&self) -> Result<u32> { match self { Value::Int(i) => Ok((*i).max(0) as u32), _ => bail!("expected int") } }
    fn as_int_i32(&self) -> Result<i32> { match self { Value::Int(i) => Ok(*i as i32), _ => bail!("expected int") } }
}
