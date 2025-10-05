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
}