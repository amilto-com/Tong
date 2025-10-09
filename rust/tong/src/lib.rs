//! Tong Language Core Library
//! 
//! This library exposes the lexer, parser, and runtime for the Tong programming language.
//! It can be compiled to WebAssembly for browser-based execution.

pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod execute;
pub mod eval;
pub mod value;
pub mod env;
pub mod builtins;
pub mod repl;
pub mod args;
pub mod sdl;
pub mod linalg;
pub mod wasm_backend;

// Re-export commonly used types
pub use lexer::lex;
pub use parser::{parse, Program};
pub use runtime::Repl;
pub use execute::{execute_with_cli, builtin_functions, builtin_modules};
pub use value::Value;

/// Compile Tong source code to WebAssembly binary
pub fn compile_to_wasm(source: &str) -> anyhow::Result<Vec<u8>> {
    let tokens = lex(source)?;
    let program = parse(tokens)?;
    wasm_backend::compile_to_wasm(&program)
}

/// Compile Tong source code to WAT (WebAssembly Text format)
pub fn compile_to_wat(source: &str) -> anyhow::Result<String> {
    let tokens = lex(source)?;
    let program = parse(tokens)?;
    wasm_backend::compile_to_wat(&program)
}

/// Execute Tong source code and return the result
pub fn execute(source: &str) -> anyhow::Result<()> {
    let tokens = lex(source)?;
    let program = parse(tokens)?;
    execute_with_cli(program, false, None, vec![])
}

/// Execute Tong source code with debug output
pub fn execute_debug(source: &str) -> anyhow::Result<()> {
    let tokens = lex(source)?;
    let program = parse(tokens)?;
    execute_with_cli(program, true, None, vec![])
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct TongRepl {
    repl: Repl,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl TongRepl {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Set panic hook for better error messages in browser
        console_error_panic_hook::set_once();
        Self {
            repl: Repl::new(),
        }
    }

    /// Evaluate a line of Tong code and return the result as a string
    #[wasm_bindgen]
    pub fn eval(&mut self, source: &str) -> Result<String, JsValue> {
        match self.repl.eval_snippet(source) {
            Ok(Some(s)) => Ok(s),
            Ok(None) => Ok(String::new()),
            Err(e) => Err(JsValue::from_str(&format!("Error: {}", e))),
        }
    }

    /// Compile source to WASM and return as Uint8Array
    #[wasm_bindgen]
    pub fn compile_wasm(source: &str) -> Result<Vec<u8>, JsValue> {
        compile_to_wasm(source)
            .map_err(|e| JsValue::from_str(&format!("Compilation error: {}", e)))
    }

    /// Compile source to WAT text format
    #[wasm_bindgen]
    pub fn compile_wat(source: &str) -> Result<String, JsValue> {
        compile_to_wat(source)
            .map_err(|e| JsValue::from_str(&format!("Compilation error: {}", e)))
    }
}
