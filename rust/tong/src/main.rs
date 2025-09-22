use clap::Parser;
use std::fs;

mod lexer;
mod parser;
mod runtime;

#[derive(Parser)]
#[command(name = "tong")] 
#[command(version = "0.1.0")] 
#[command(about = "TONG - The Ultimate Programming Language (Rust MVP)")]
struct Cli {
    /// Path to a .tong source file to run
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
        println!("TONG (Rust MVP) - run a .tong file, e.g.: cargo run -- ../../examples/hello.tong");
    }

    Ok(())
}
