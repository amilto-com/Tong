use super::*;

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
            "args_len" => Ok(Value::Int(self.cli_args.len() as i64)),
            "args_get" => {
                if values.len() != 1 {
                    bail!("args.get expects 1 argument");
                }
                if let Value::Int(i) = values[0] {
                    if i >= 0 && (i as usize) < self.cli_args.len() {
                        Ok(Value::Str(self.cli_args[i as usize].clone()))
                    } else {
                        Ok(Value::Str("".to_string()))
                    }
                } else {
                    bail!("args.get expects int index");
                }
            }
            "args_has" => {
                if values.len() != 1 {
                    bail!("args.has expects 1 argument");
                }
                if let Value::Str(s) = &values[0] {
                    Ok(Value::Bool(self.cli_args.contains(s)))
                } else {
                    bail!("args.has expects string");
                }
            }
            "args_value" => {
                if values.len() != 1 {
                    bail!("args.value expects 1 argument");
                }
                if let Value::Str(s) = &values[0] {
                    if let Some(pos) = self.cli_args.iter().position(|a| a == s) {
                        if pos + 1 < self.cli_args.len() {
                            Ok(Value::Str(self.cli_args[pos + 1].clone()))
                        } else {
                            Ok(Value::Str("".to_string()))
                        }
                    } else {
                        Ok(Value::Str("".to_string()))
                    }
                } else {
                    bail!("args.value expects string");
                }
            }
            "args_parse_int" => {
                if values.len() != 1 {
                    bail!("args.parse_int expects 1 argument");
                }
                if let Value::Str(s) = &values[0] {
                    if let Ok(i) = s.parse::<i64>() {
                        Ok(Value::Int(i))
                    } else {
                        Ok(Value::Int(0))
                    }
                } else {
                    bail!("args.parse_int expects string");
                }
            }
            _ => bail!("unknown args builtin {}", name),
        }
    }
}