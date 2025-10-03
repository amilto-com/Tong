use clap::Parser;
use std::fs;

mod lexer;
mod parser;
mod runtime;
use runtime::{builtin_functions, builtin_modules, Repl};

#[derive(Parser)]
#[command(name = "tong")]
#[command(version = "0.1.0")]
#[command(
    about = "TONG - The Ultimate Programming Language (Rust MVP). Run with a .tong file to execute it, or with no arguments to start the interactive REPL."
)]
struct Cli {
    /// Path to a .tong source file to run (if omitted, starts interactive REPL)
    file: Option<String>,
    /// List built-in modules and exit
    #[arg(long)]
    modules: bool,
    /// Show extended version (git hash, build timestamp) and exit
    #[arg(long)]
    version_long: bool,
    /// List core built-in functions and exit
    #[arg(long)]
    list_builtins: bool,
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
        let src = fs::read_to_string(&file)?;
        let tokens = lexer::lex(&src)?;
        let program = parser::parse(tokens)?;
        runtime::execute(program)?;
    } else {
        // Interactive REPL
        println!("TONG REPL - type :help for commands, :quit to exit");
        let mut repl = Repl::new();
        use std::io::{self, Write};
        let mut buffer = String::new();
        let mut open_braces: i32 = 0;
        loop {
            let prompt = if open_braces > 0 { "...." } else { "tong>" };
            print!("{} ", prompt);
            io::stdout().flush().ok();
            let mut line = String::new();
            if io::stdin().read_line(&mut line)? == 0 {
                break;
            }
            let trimmed = line.trim_end();
            if open_braces == 0
                && (trimmed.starts_with(':') || matches!(trimmed, "quit" | "q" | "exit"))
            {
                match trimmed {
                    ":quit" | ":q" | ":exit" => break,
                    "quit" | "q" | "exit" => break,
                    ":help" => {
                        println!(":quit/:q exit | :reset clear state | :env list vars | :modules list built-in modules | multi-line blocks supported (balanced {{ }})");
                    }
                    ":env" => {
                        for (k, v) in repl.list_vars() {
                            println!("{} = {}", k, v);
                        }
                    }
                    ":modules" => {
                        let mods = builtin_modules().join(", ");
                        println!("Built-in modules: {}", mods);
                    }
                    ":reset" => {
                        repl.reset();
                        println!("(state cleared)");
                    }
                    other => println!("Unknown command {}", other),
                }
                continue;
            }
            // naive brace balance (ignore strings)
            for ch in trimmed.chars() {
                if ch == '{' {
                    open_braces += 1;
                } else if ch == '}' {
                    open_braces -= 1;
                }
            }
            buffer.push_str(&line);
            if open_braces <= 0 && !buffer.trim().is_empty() {
                match repl.eval_snippet(&buffer) {
                    Ok(Some(val)) => println!("{}", val),
                    Ok(None) => {}
                    Err(e) => println!("Error: {}", e),
                }
                buffer.clear();
                open_braces = 0;
            }
        }
    }

    Ok(())
}
