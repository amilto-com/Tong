use clap::Parser;
use std::fs;

mod lexer;
mod parser;
mod runtime;
use runtime::Repl;

#[derive(Parser)]
#[command(name = "tong")]
#[command(version = "0.1.0")]
#[command(about = "TONG - The Ultimate Programming Language (Rust MVP). Run with a .tong file to execute it, or with no arguments to start the interactive REPL.")]
struct Cli {
    /// Path to a .tong source file to run (if omitted, starts interactive REPL)
    file: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

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
            if io::stdin().read_line(&mut line)? == 0 { break; }
            let trimmed = line.trim_end();
            if open_braces == 0 && (trimmed.starts_with(':') || matches!(trimmed, "quit"|"q"|"exit")) {
                match trimmed {
                    ":quit" | ":q" | ":exit" => break,
                    "quit" | "q" | "exit" => break,
                    ":help" => {
                        println!(":quit/:q  exit  | :reset clear state | :env list vars | multi-line blocks supported (balanced {{ }})");
                    }
                    ":env" => {
                        for (k, v) in repl.list_vars() { println!("{} = {}", k, v); }
                    }
                    ":reset" => { repl.reset(); println!("(state cleared)"); }
                    other => println!("Unknown command {}", other),
                }
                continue;
            }
            // naive brace balance (ignore strings)
            for ch in trimmed.chars() {
                if ch == '{' { open_braces += 1; }
                else if ch == '}' { open_braces -= 1; }
            }
            buffer.push_str(&line);
            if open_braces <= 0 && !buffer.trim().is_empty() {
                match repl.eval_snippet(&buffer) {
                    Ok(Some(val)) => println!("{}", val),
                    Ok(None) => {},
                    Err(e) => println!("Error: {}", e),
                }
                buffer.clear();
                open_braces = 0;
            }
        }
    }

    Ok(())
}
