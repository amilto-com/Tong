use clap::Parser;
use std::fs;

mod lexer;
mod parser;
mod runtime;
mod execute;
mod eval;
mod value;
mod env;
mod builtins;
mod repl;
mod args;
mod sdl;
mod linalg;
mod wasm_backend;

use execute::{builtin_functions, builtin_modules, execute_with_cli};
use runtime::Repl;
use rustyline::DefaultEditor;

#[derive(Parser)]
#[command(name = "tong")]
#[command(version = concat!(env!("CARGO_PKG_VERSION"), "-", env!("BUILD_TIME")))]
#[command(
    about = "TONG - The Ultimate Programming Language (Rust MVP). Run with a .tong file to execute it, or with no arguments to start the interactive REPL."
)]
struct Cli {
    /// Path to a .tong source file to run (if omitted, starts interactive REPL)
    file: Option<String>,
    /// Arguments passed to the script (after the file). These may start with '-' and are not parsed by tong itself.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    script_args: Vec<String>,
    /// List built-in modules and exit
    #[arg(long)]
    modules: bool,
    /// Show extended version (git hash, build timestamp) and exit
    #[arg(long)]
    version_long: bool,
    /// List core built-in functions and exit
    #[arg(long)]
    list_builtins: bool,
    /// Enable verbose runtime debug tracing (statement exec, function calls, SDL events)
    #[arg(short, long)]
    debug: bool,
    /// Print explicit exit status line on completion
    #[arg(long)]
    show_exit: bool,
    /// Compile to WASM instead of executing (outputs .wasm file)
    #[arg(long)]
    wasm: bool,
    /// Compile to WAT (WebAssembly Text format) instead of executing
    #[arg(long)]
    wat: bool,
    /// Output file for WASM/WAT compilation (default: <input>.wasm or <input>.wat)
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.version_long {
        let hash = option_env!("GIT_HASH").unwrap_or("unknown");
        let dirty = option_env!("GIT_DIRTY").unwrap_or("unknown");
        let ts = option_env!("BUILD_UNIX").unwrap_or("0");
        println!(
            "tong {} (hash:{} {} build_ts:{})",
            env!("CARGO_PKG_VERSION"),
            hash,
            dirty,
            ts
        );
        return Ok(());
    }

    if cli.list_builtins {
        let funcs = builtin_functions().join(", ");
        println!("Built-in functions: {}", funcs);
        return Ok(());
    }

    if cli.modules {
        let mods = builtin_modules().join(", ");
        println!("Built-in modules: {}", mods);
        return Ok(());
    }

    if let Some(file) = cli.file {
        let result = (|| -> anyhow::Result<()> {
            let src = fs::read_to_string(&file)?;
            let tokens = lexer::lex(&src)?;
            let program = parser::parse(tokens)?;
            
            // Handle WASM/WAT compilation
            if cli.wasm || cli.wat {
                let output_path = if let Some(out) = cli.output {
                    out
                } else {
                    let base = file.trim_end_matches(".tong");
                    if cli.wat {
                        format!("{}.wat", base)
                    } else {
                        format!("{}.wasm", base)
                    }
                };

                if cli.wat {
                    // Compile to WAT
                    let wat = wasm_backend::compile_to_wat(&program)?;
                    fs::write(&output_path, wat)?;
                    println!("Compiled to WAT: {}", output_path);
                } else {
                    // Compile to WASM
                    let wasm = wasm_backend::compile_to_wasm(&program)?;
                    fs::write(&output_path, wasm)?;
                    println!("Compiled to WASM: {}", output_path);
                }
            } else {
                // Normal execution
                // Propagate script path and CLI args into runtime ENV via globals
                execute_with_cli(program, cli.debug, Some(file), cli.script_args)?;
            }
            Ok(())
        })();
        match result {
            Ok(()) => {
                if cli.debug || cli.show_exit {
                    eprintln!("[TONG][exit] success");
                }
            }
            Err(e) => {
                eprintln!("[TONG][exit][error] {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Interactive REPL
        println!("TONG REPL - type :help for commands, :quit to exit");
        let mut repl = Repl::new();
        let mut rl = DefaultEditor::new().unwrap();
        loop {
            let readline = rl.readline("tong> ");
            match readline {
                Ok(line) => {
                    let _ = rl.add_history_entry(line.as_str());
                    let src = line.trim();
                    if src.is_empty() { continue; }
                    if src == ":quit" { break; }
                    if src == ":help" {
                        println!("Commands: :quit, :help");
                        continue;
                    }
                    match repl.eval_snippet(src) {
                        Ok(Some(s)) => println!("{}", s),
                        Ok(None) => {},
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                Err(_) => break,
            }
        }
    }

    Ok(())
}
