use anyhow::Result;

use crate::env::Env;
use crate::lexer::lex;
use crate::parser::{parse, Stmt};
use crate::value::Value;
use rustyline::{Editor, Result as RustyResult};

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
        let tokens = lex(src)?;
        let program = parse(tokens)?;

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
                Stmt::FnDefGuardedTyped(name, params, ret_ann, guard, body) => {
                    self.env.fn_types.insert(
                        name.clone(),
                        (
                            params.iter().map(|(_, t)| t.clone()).collect(),
                            ret_ann.clone(),
                        ),
                    );
                    self.env
                        .guarded_funcs
                        .entry(name.clone())
                        .or_default()
                        .push((
                            params.iter().map(|(n, _)| n.clone()).collect(),
                            guard.clone(),
                            body.clone(),
                        ));
                }
                Stmt::FnDefPattern(name, patterns, guard, ret_ann, body) => {
                    // Append pattern clause maintaining order of entry across snippets
                    self.env
                        .pattern_funcs
                        .entry(name.clone())
                        .or_default()
                        .push((
                            patterns.clone(),
                            guard.clone(),
                            ret_ann.clone(),
                            body.clone(),
                        ));
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
                    self.env.declare_let(name.clone(), v);
                }
                Stmt::Assign(name, expr) => {
                    let v = self.env.eval_expr(expr.clone())?;
                    if self.env.vars().contains_key(name) {
                        self.env.assign_var(name, v)?;
                    } else {
                        self.env.declare_var(name.clone(), v);
                    }
                }
                Stmt::Var(name, expr) => {
                    let v = self.env.eval_expr(expr.clone())?;
                    self.env.declare_var(name.clone(), v);
                }
                Stmt::VarAnn(name, _, expr) => {
                    let v = self.env.eval_expr(expr.clone())?;
                    self.env.declare_var(name.clone(), v);
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
                        Stmt::If(..) | Stmt::While(..) | Stmt::Parallel(..) | Stmt::Return(..) | Stmt::Var(..) | Stmt::ArrayAssign(..) => {
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