use clap::Parser;
use std::fs;

mod lexer;
mod parser;
mod runtime;
mod value;
mod env;
mod builtins;
mod repl;
mod args;
mod sdl;
mod linalg;

use runtime::{builtin_functions, builtin_modules, Repl};
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
            // Propagate script path and CLI args into runtime ENV via globals
            runtime::execute_with_cli(program, cli.debug, Some(file), cli.script_args)?;
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
