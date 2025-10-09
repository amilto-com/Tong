use anyhow::{bail, Result};
use std::collections::HashMap;
use crate::value::Value;
use crate::env::Env;

impl Env {
    pub fn import_args(&mut self) -> Value {
        let mut obj = HashMap::new();
        // properties
        let script = self.cli_script.clone().unwrap_or_default();
        obj.insert("script".to_string(), Value::Str(script.clone()));
        let args_arr = Value::Array(self.cli_args.iter().cloned().map(Value::Str).collect());
        obj.insert("args".to_string(), args_arr);
        let mut all_vec: Vec<Value> = Vec::new();
        if !script.is_empty() {
            all_vec.push(Value::Str(script));
        }
        for a in &self.cli_args {
            all_vec.push(Value::Str(a.clone()));
        }
        obj.insert("all".to_string(), Value::Array(all_vec));
        // methods
        obj.insert("len".into(), Value::FuncRef("args_len".into()));
        obj.insert("get".into(), Value::FuncRef("args_get".into()));
        obj.insert("has".into(), Value::FuncRef("args_has".into()));
        obj.insert("value".into(), Value::FuncRef("args_value".into()));
        obj.insert("parse_int".into(), Value::FuncRef("args_parse_int".into()));
        Value::Object(obj)
    }

    pub fn call_args_builtin_values(&mut self, name: &str, values: Vec<Value>) -> Result<Value> {
        match name {
            "args_script" => Ok(Value::Str(self.cli_script.clone().unwrap_or_default())),
            "args_len" => Ok(Value::Int(self.cli_args.len() as i64)),
            "args_all" => Ok(Value::Array(self.cli_args.iter().map(|s| Value::Str(s.clone())).collect())),
            "args_has" => {
                if values.len() != 1 {
                    bail!("args.has expects 1 argument");
                }
                match &values[0] {
                    Value::Str(flag) => Ok(Value::Bool(self.cli_args.contains(flag))),
                    _ => bail!("args.has expects string"),
                }
            }
            "args_value" => {
                if values.len() != 1 {
                    bail!("args.value expects 1 argument");
                }
                match &values[0] {
                    Value::Str(key) => {
                        for arg in &self.cli_args {
                            if arg.starts_with(&format!("{}=", key)) {
                                return Ok(Value::Str(arg[key.len() + 1..].to_string()));
                            }
                        }
                        Ok(Value::Str(String::new()))
                    }
                    _ => bail!("args.value expects string"),
                }
            }
            "args_get" => {
                if values.len() != 1 {
                    bail!("args.get expects 1 argument");
                }
                match &values[0] {
                    Value::Int(idx) if *idx >= 0 => {
                        let i = *idx as usize;
                        if i < self.cli_args.len() {
                            Ok(Value::Str(self.cli_args[i].clone()))
                        } else {
                            Ok(Value::Str(String::new()))
                        }
                    }
                    _ => bail!("args.get expects non-negative integer"),
                }
            }
            _ => bail!("unknown args function {}", name),
        }
    }
}