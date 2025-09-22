use anyhow::{bail, Result};
use std::collections::HashMap;

use crate::parser::{BinOp, Expr, Program, Stmt};

pub fn execute(program: Program) -> Result<()> {
    let mut env = Env::new();

    // First collect function definitions
    for stmt in &program.stmts {
        if let Stmt::FnDef(name, params, body) = stmt { env.funcs.insert(name.clone(), (params.clone(), body.clone())); }
        if let Stmt::FnMain(body) = stmt { env.funcs.insert("main".to_string(), (Vec::new(), body.clone())); }
    }

    // Execute top-level statements (let/assign/print)
    for stmt in &program.stmts {
        match stmt {
            Stmt::Let(name, expr) => { let v = env.eval_expr(expr.clone())?; env.vars_mut().insert(name.clone(), v); }
            Stmt::Assign(name, expr) => { let v = env.eval_expr(expr.clone())?; env.vars_mut().insert(name.clone(), v); }
            Stmt::Print(args) => { let parts: Result<Vec<String>> = args.iter().cloned().map(|e| env.eval_expr(e).map(|v| format_value(&v))).collect(); println!("{}", parts?.join(" ")); }
            Stmt::Expr(e) => { let _ = env.eval_expr(e.clone())?; }
            _ => {}
        }
    }

    // Call main() if present
    if env.funcs.contains_key("main") {
        let _ = env.call_function("main".to_string(), vec![])?;
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
    Lambda { params: Vec<String>, body: Box<Expr>, env: HashMap<String, Value> },
    FuncRef(String),
}

#[derive(Default)]
struct Env {
    vars_stack: Vec<HashMap<String, Value>>, // lexical-style stack
    funcs: HashMap<String, (Vec<String>, Vec<Stmt>)>,
}

impl Env {
    fn new() -> Self { Self { vars_stack: vec![HashMap::new()], funcs: HashMap::new() } }
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
                                    self.call_lambda(params, *body, env, evaled?.into_iter().map(|v| expr_from_value(&v)).collect())?
                                }
                                Value::FuncRef(name) => {
                                    let evaled: Result<Vec<Value>> = args.into_iter().map(|a| self.eval_expr(a)).collect();
                                    self.call_function(name, evaled?.into_iter().map(|v| expr_from_value(&v)).collect())?
                                }
                                _ => bail!("{} is not callable", callee),
                            }
                        } else if self.funcs.contains_key(&callee) {
                            let evaled: Result<Vec<Value>> = args.into_iter().map(|a| self.eval_expr(a)).collect();
                            self.call_function(callee.clone(), evaled?.into_iter().map(|v| expr_from_value(&v)).collect())?
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

                    (Value::Float(a), Value::Float(b), BinOp::Eq) => Value::Bool((a - b).abs() < std::f64::EPSILON),
                    (Value::Int(a), Value::Int(b), BinOp::Eq) => Value::Bool(a == b),
                    (Value::Bool(a), Value::Bool(b), BinOp::Eq) => Value::Bool(a == b),
                    (Value::Str(a), Value::Str(b), BinOp::Eq) => Value::Bool(a == b),

                    (Value::Float(a), Value::Float(b), BinOp::Ne) => Value::Bool((a - b).abs() >= std::f64::EPSILON),
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
                        Value::Lambda { params, body, env } => self.call_lambda(params, *body, env, args),
                        Value::FuncRef(fname) => self.call_function(fname, args),
                        _ => bail!("{} is not callable", name),
                    }
                } else if self.funcs.contains_key(&name) {
                    self.call_function(name, args)
                } else {
                    bail!("unknown function {}", name)
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
    fn call_function(&mut self, name: String, args: Vec<Expr>) -> Result<Value> {
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

        // Execute body
        let mut ret: Option<Value> = None;
        for s in body {
            match s {
                Stmt::Let(n, e) => { let v = self.eval_expr(e)?; self.vars_mut().insert(n, v); }
                Stmt::Assign(n, e) => { let v = self.eval_expr(e)?; self.vars_mut().insert(n, v); }
                Stmt::Print(args) => { let mut parts = Vec::new(); for e in args { parts.push(format_value(&self.eval_expr(e)?)); } println!("{}", parts.join(" ")); }
                Stmt::Return(e) => { ret = Some(self.eval_expr(e)?); break; }
                Stmt::FnDef(name, params, body) => { self.funcs.insert(name, (params, body)); }
                Stmt::Parallel(inner) => {
                    // No-op for now: execute sequentially
                    for is in inner {
                        match is {
                            Stmt::Let(n, e) => { let v = self.eval_expr(e)?; self.vars_mut().insert(n, v); }
                            Stmt::Assign(n, e) => { let v = self.eval_expr(e)?; self.vars_mut().insert(n, v); }
                            Stmt::Print(args) => { let mut parts = Vec::new(); for e in args { parts.push(format_value(&self.eval_expr(e)?)); } println!("{}", parts.join(" ")); }
                            Stmt::Return(e) => { ret = Some(self.eval_expr(e)?); }
                            Stmt::FnDef(name, params, body) => { self.funcs.insert(name, (params, body)); }
                            Stmt::If(_, _, _) | Stmt::Expr(_) | Stmt::FnMain(_) | Stmt::Parallel(_) | Stmt::While(_, _) => { /* ignore here, outer loop will handle */ }
                        }
                        if ret.is_some() { break; }
                    }
                }
                Stmt::While(cond, body) => {
                    while matches!(self.eval_expr(cond.clone())?, Value::Bool(true)) {
                        for ws in body.clone() {
                            match ws {
                                Stmt::Let(n, e) => { let v = self.eval_expr(e)?; self.vars_mut().insert(n, v); }
                                Stmt::Assign(n, e) => { let v = self.eval_expr(e)?; self.vars_mut().insert(n, v); }
                                Stmt::Print(args) => { let mut parts = Vec::new(); for e in args { parts.push(format_value(&self.eval_expr(e)?)); } println!("{}", parts.join(" ")); }
                                Stmt::Return(e) => { ret = Some(self.eval_expr(e)?); break; }
                                Stmt::FnDef(name, params, body) => { self.funcs.insert(name, (params, body)); }
                                Stmt::If(_, _, _) | Stmt::Expr(_) | Stmt::FnMain(_) | Stmt::Parallel(_) | Stmt::While(_, _) => { /* ignore */ }
                            }
                            if ret.is_some() { break; }
                        }
                        if ret.is_some() { break; }
                    }
                }
                Stmt::If(cond, then_body, else_body) => {
                    let v = self.eval_expr(cond)?;
                    if matches!(v, Value::Bool(true)) {
                        for ts in then_body {
                            match ts {
                                Stmt::Let(n, e) => { let v = self.eval_expr(e)?; self.vars_mut().insert(n, v); }
                                Stmt::Assign(n, e) => { let v = self.eval_expr(e)?; self.vars_mut().insert(n, v); }
                                Stmt::Print(args) => { let mut parts = Vec::new(); for e in args { parts.push(format_value(&self.eval_expr(e)?)); } println!("{}", parts.join(" ")); }
                                Stmt::Return(e) => { ret = Some(self.eval_expr(e)?); break; }
                                Stmt::FnDef(name, params, body) => { self.funcs.insert(name, (params, body)); }
                                Stmt::If(_, _, _) | Stmt::Expr(_) | Stmt::FnMain(_) | Stmt::While(_, _) | Stmt::Parallel(_) => { /* ignore */ }
                            }
                            if ret.is_some() { break; }
                        }
                    } else if let Some(else_body) = else_body {
                        for ts in else_body {
                            match ts {
                                Stmt::Let(n, e) => { let v = self.eval_expr(e)?; self.vars_mut().insert(n, v); }
                                Stmt::Assign(n, e) => { let v = self.eval_expr(e)?; self.vars_mut().insert(n, v); }
                                Stmt::Print(args) => { let mut parts = Vec::new(); for e in args { parts.push(format_value(&self.eval_expr(e)?)); } println!("{}", parts.join(" ")); }
                                Stmt::Return(e) => { ret = Some(self.eval_expr(e)?); break; }
                                Stmt::FnDef(name, params, body) => { self.funcs.insert(name, (params, body)); }
                                Stmt::If(_, _, _) | Stmt::Expr(_) | Stmt::FnMain(_) | Stmt::While(_, _) | Stmt::Parallel(_) => { /* ignore */ }
                            }
                            if ret.is_some() { break; }
                        }
                    }
                }
                Stmt::Expr(e) => { let _ = self.eval_expr(e)?; }
                Stmt::FnMain(_) => {}
            }
            if ret.is_some() { break; }
        }

        // Pop the frame
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
    }
}
