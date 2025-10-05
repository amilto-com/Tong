use anyhow::{bail, Result};
use std::collections::HashMap;
use crate::value::Value;
use crate::env::Env;

impl Env {
    pub fn import_linalg(&mut self) -> Value {
        let obj = HashMap::new();
        // TODO: implement linalg module
        Value::Object(obj)
    }

    pub fn call_linalg_builtin_values(&mut self, name: &str, _values: Vec<Value>) -> Result<Value> {
        match name {
            // TODO: implement linalg builtins
            _ => bail!("unknown linalg builtin {}", name),
        }
    }
}