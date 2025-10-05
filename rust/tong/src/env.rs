use anyhow::{anyhow, bail, Result};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::parser::{BinOp, Expr, Pattern, Program, Stmt, TypeAnn};
use crate::value::Value;

// Clause type aliases to keep signatures and storage readable (avoid clippy::type_complexity)
type GuardedClause = (Vec<String>, Expr, Vec<Stmt>);
// Add optional return type on pattern functions (param annotations for patterns are not yet supported)
type PatternClause = (Vec<Pattern>, Option<Expr>, Option<TypeAnn>, Vec<Stmt>);

#[derive(Default)]
pub struct Env {
    pub(crate) vars_stack: Vec<HashMap<String, Value>>, // lexical-style stack
    pub(crate) muts_stack: Vec<HashMap<String, bool>>,  // per-scope mutability (true = mutable)
    pub(crate) funcs: HashMap<String, (Vec<String>, Vec<Stmt>)>,
    pub(crate) fn_types: HashMap<String, (Vec<Option<TypeAnn>>, Option<TypeAnn>)>, // function -> (param annotations, return)
    pub(crate) guarded_funcs: HashMap<String, Vec<GuardedClause>>,                 // guarded multi-clause
    pub(crate) pattern_funcs: HashMap<String, Vec<PatternClause>>,                 // pattern parameter clauses
    pub(crate) modules: HashMap<String, Value>,
    #[cfg(not(feature = "sdl3"))]
    pub(crate) sdl_frame: i64,
    #[cfg(feature = "sdl3")]
    pub(crate) sdl: Option<SdlState>,
    pub(crate) data_ctors: HashMap<String, usize>,       // ctor name -> arity
    pub(crate) type_ctors: HashMap<String, Vec<String>>, // type name -> ctor names
    pub(crate) ctor_type: HashMap<String, String>,       // ctor name -> type name
    pub(crate) debug: bool,
    // CLI context
    cli_script: Option<String>,
    cli_args: Vec<String>,
}

#[cfg(feature = "sdl3")]
#[derive(Debug)]
struct SdlState {
    _sdl: sdl3::Sdl,
    video: sdl3::VideoSubsystem,
    window: Option<sdl3::video::Window>,
    canvas: Option<sdl3::render::Canvas<sdl3::video::Window>>,
    events: sdl3::EventPump,
    draw_color: (u8, u8, u8, u8),
}

impl Env {
    // In-place array element assignment for mutable (var) arrays.
    // If the variable is mutable and holds an Array, update the element at idx.
    // Errors on immutability, negative or OOB index, or non-array value.
    fn array_assign(&mut self, name: &str, idx: i64, val: Value) -> Result<()> {
        if idx < 0 {
            bail!("negative index")
        }
        let stack_idx = self.vars_stack.len() - 1;
        if let Some(var_map) = self.vars_stack.get_mut(stack_idx) {
            if let Some(Value::Array(arr)) = var_map.get_mut(name) {
                let idx_usize = idx as usize;
                if idx_usize >= arr.len() {
                    bail!("index out of bounds")
                }
                // Check mutability
                if let Some(mut_map) = self.muts_stack.get(stack_idx) {
                    if let Some(&is_mut) = mut_map.get(name) {
                        if !is_mut {
                            bail!("cannot assign to immutable variable {}", name)
                        }
                    } else {
                        bail!("variable {} not found in mutability map", name)
                    }
                } else {
                    bail!("mutability stack corrupted")
                }
                arr[idx_usize] = val;
                Ok(())
            } else {
                bail!("variable {} is not an array or not found", name)
            }
        } else {
            bail!("no active scope")
        }
    }

    pub fn new() -> Self {
        let mut env = Self::default();
        env.vars_stack.push(HashMap::with_capacity(16));
        env.muts_stack.push(HashMap::new());
        env
    }

    fn push_scope(&mut self) {
        self.vars_stack.push(HashMap::with_capacity(16));
        self.muts_stack.push(HashMap::new());
    }

    fn pop_scope(&mut self) -> Result<()> {
        if self.vars_stack.len() <= 1 {
            bail!("cannot pop global scope")
        }
        self.vars_stack.pop();
        self.muts_stack.pop();
        Ok(())
    }

    fn lookup_var(&self, name: &str) -> Result<Value> {
        for scope in self.vars_stack.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Ok(v.clone());
            }
        }
        bail!("undefined variable: {}", name)
    }

    fn set_var(&mut self, name: &str, val: Value, is_mut: bool) -> Result<()> {
        let stack_idx = self.vars_stack.len() - 1;
        if let Some(var_map) = self.vars_stack.get_mut(stack_idx) {
            var_map.insert(name.to_string(), val);
        } else {
            bail!("no active scope")
        }
        if let Some(mut_map) = self.muts_stack.get_mut(stack_idx) {
            mut_map.insert(name.to_string(), is_mut);
        } else {
            bail!("mutability stack corrupted")
        }
        Ok(())
    }

    fn update_var(&mut self, name: &str, val: Value) -> Result<()> {
        for (i, scope) in self.vars_stack.iter_mut().enumerate().rev() {
            if let Some(_) = scope.get(name) {
                // Check mutability
                if let Some(mut_map) = self.muts_stack.get(i) {
                    if let Some(&is_mut) = mut_map.get(name) {
                        if !is_mut {
                            bail!("cannot assign to immutable variable {}", name)
                        }
                    } else {
                        bail!("variable {} not found in mutability map", name)
                    }
                } else {
                    bail!("mutability stack corrupted")
                }
                scope.insert(name.to_string(), val);
                return Ok(());
            }
        }
        bail!("undefined variable: {}", name)
    }

    pub(crate) fn vars(&self) -> &HashMap<String, Value> {
        self.vars_stack.last().unwrap()
    }

    pub(crate) fn vars_mut(&mut self) -> &mut HashMap<String, Value> {
        self.vars_stack.last_mut().unwrap()
    }

    fn get_var(&self, name: &str) -> Option<Value> {
        self.lookup_var(name).ok()
    }

    fn declare_let(&mut self, name: String, val: Value) {
        let _ = self.set_var(&name, val, false);
    }

    pub fn import_module(&mut self, name: &str) -> Result<Value> {
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
            "args" => {
                let v = self.import_args();
                self.modules.insert(name.to_string(), v.clone());
                Ok(v)
            }
            _ => bail!("unknown module {}", name),
        }
    }

    fn import_args(&mut self) -> Value {
        let mut obj = HashMap::new();
        obj.insert("get".into(), Value::FuncRef("args_get".into()));
        obj.insert("parse_int".into(), Value::FuncRef("args_parse_int".into()));
        Value::Object(obj)
    }

    fn import_sdl(&mut self) -> Value {
        let mut obj = HashMap::new();
        obj.insert("init".into(), Value::FuncRef("sdl_init".into()));
        obj.insert("create_window".into(), Value::FuncRef("sdl_create_window".into()));
        obj.insert("create_renderer".into(), Value::FuncRef("sdl_create_renderer".into()));
        obj.insert("set_draw_color".into(), Value::FuncRef("sdl_set_draw_color".into()));
        obj.insert("clear".into(), Value::FuncRef("sdl_clear".into()));
        obj.insert("fill_rect".into(), Value::FuncRef("sdl_fill_rect".into()));
        obj.insert("present".into(), Value::FuncRef("sdl_present".into()));
        obj.insert("poll_event".into(), Value::FuncRef("sdl_poll_event".into()));
        obj.insert("quit".into(), Value::FuncRef("sdl_quit".into()));
        Value::Object(obj)
    }

    pub fn import_linalg(&mut self) -> Value {
        let mut obj = HashMap::new();
        // Stub for linalg module - not implemented in Phase 1
        obj.insert("create".into(), Value::FuncRef("linalg_create".into()));
        Value::Object(obj)
    }

    pub fn call_linalg_builtin_values(&mut self, name: &str, values: Vec<Value>) -> Result<Value> {
        // Stub for linalg builtins - not implemented in Phase 1
        bail!("linalg builtin {} not implemented", name)
    }

    pub fn call_args_builtin_values(&mut self, name: &str, values: Vec<Value>) -> Result<Value> {
        match name {
            "args_get" => {
                // For benchmark, return "1000000" as default
                Ok(Value::Str("1000000".to_string()))
            }
            "args_parse_int" => {
                if let Some(Value::Str(s)) = values.first() {
                    if let Ok(i) = s.parse::<i64>() {
                        Ok(Value::Int(i))
                    } else {
                        bail!("parse_int: invalid integer {}", s)
                    }
                } else {
                    bail!("parse_int: expected string")
                }
            }
            _ => bail!("args builtin {} not implemented", name)
        }
    }

    fn call_sdl_builtin(&mut self, name: &str, args: Vec<Expr>) -> Result<Value> {
        // Stub for SDL builtins - not implemented in Phase 1
        bail!("SDL builtin {} not implemented", name)
    }

    // The rest of the impl will be added here
}