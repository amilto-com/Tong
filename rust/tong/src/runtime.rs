use anyhow::Result;

use crate::parser::{Expr, Pattern, Stmt, TypeAnn, Program};

pub use crate::value::*;
pub use crate::env::*;
pub use crate::repl::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::parse as parse_prog;

    fn run(src: &str) -> Result<Vec<String>> {
        let tokens = lex(src)?;
        let program = parse_prog(tokens)?;
        let mut env = Env::new();
        for s in &program.stmts {
            match s {
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
                Stmt::DataDecl(type_name, ctors) => {
                    let mut names = Vec::new();
                    for c in ctors {
                        env.data_ctors.insert(c.name.clone(), c.arity);
                        names.push(c.name.clone());
                        env.ctor_type.insert(c.name.clone(), type_name.clone());
                    }
                    env.type_ctors.insert(type_name.clone(), names);
                }
                _ => {}
            }
        }
        let mut outputs = Vec::new();
        for s in &program.stmts {
            match s {
                Stmt::Print(args) => {
                    let parts: Result<Vec<String>> = args
                        .iter()
                        .cloned()
                        .map(|e| env.eval_expr(e).map(|v| format_value(&v)))
                        .collect();
                    outputs.push(parts?.join(" "));
                }
                Stmt::Expr(e) => {
                    let v = env.eval_expr(e.clone())?;
                    outputs.push(format_value(&v));
                }
                _ => {
                    let _ = env.exec_stmt(s)?;
                }
            }
        }
        Ok(outputs)
    }

    #[test]
    fn pattern_fn_ctors_basic_and_guarded() {
        let src = r#"
data Maybe = Just x | Nothing

def getOrZero(Just x) -> Int { x }
def getOrZero(Nothing) -> Int { 0 }

def safeHead(Just x) if x > 0 { x }
def safeHead(Just x) if x <= 0 { 0 }
def safeHead(Nothing) { 0 }

print(getOrZero(Just(7)))
print(getOrZero(Nothing))
print(safeHead(Just(5)))
print(safeHead(Just(0)))
print(safeHead(Nothing))
"#;
        let out = run(src).expect("program should run");
        assert_eq!(out, vec!["7", "0", "5", "0", "0"]);
    }

    #[test]
    fn pattern_fn_nested_constructors() {
        let src = r#"
data List = Cons head tail | Nil

def list_len(Cons _ t) -> Int { 1 + list_len(t) }
def list_len(Nil) -> Int { 0 }

let l = Cons(1, Cons(2, Cons(3, Nil)))
print(list_len(l))
"#;
        let out = run(src).expect("program should run");
        assert_eq!(out, vec!["3"]);
    }
}
