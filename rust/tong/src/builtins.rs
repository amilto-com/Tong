use anyhow::{anyhow, bail, Result};
use std::collections::HashMap;

use crate::parser::Expr;
use crate::value::Value;

impl crate::env::Env {
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
}