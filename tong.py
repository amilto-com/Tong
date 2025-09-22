#!/usr/bin/env python3
"""
DEPRECATED: The Python implementation of TONG has been retired.

TONG has migrated to a Rust implementation. This Python entrypoint remains only
for archival purposes and will exit immediately. Please use the Rust CLI:

  - Run an example:
      cd rust/tong && cargo run -- ../../examples/hello.tong

  - Build a release binary:
      cd rust/tong && cargo build --release

  - Install a 'tong' shim on Windows PowerShell:
      pwsh ./setup.ps1 -Global

  - Then run:
      tong path/to/program.tong
"""

import sys

msg = (
    "TONG Python is deprecated. Use the Rust CLI instead.\n"
    "See README.md for Rust-only instructions.\n"
)
print(msg)
sys.exit(1)

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Optional

# Add src to path for imports - must be done before importing from src
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'src'))

# Now import from src modules
from src.ast_dump import ast_to_dict  # pylint: disable=wrong-import-position
from src.interpreter import TongInterpreter  # pylint: disable=wrong-import-position
from src.lexer import TongLexer  # pylint: disable=wrong-import-position
from src.parser import TongParser, ParseError  # pylint: disable=wrong-import-position
from src.repl import TongREPL  # pylint: disable=wrong-import-position

def compile_file(
    filename: str,
    output: Optional[str] = None,
    print_ast: bool = False,
    save_ast: Optional[str] = None,
    compile_only: bool = False,
) -> None:
    """Compile a TONG source file"""
    try:
        with open(filename, 'r', encoding='utf-8') as f:
            source = f.read()

        print(f"Compiling {filename}...")

        # Tokenize
        lexer = TongLexer(source)
        tokens = lexer.tokenize()
        print(f"  Lexical analysis: {len(tokens)} tokens")

        # Parse
        parser = TongParser(tokens)
        program = parser.parse()
        print(f"  Parsing: {len(program.statements)} statements")

        # Optionally print or save the AST (JSON)
        if print_ast or save_ast:
            ast_json = json.dumps(ast_to_dict(program), indent=2)
            if print_ast:
                print("=== AST (JSON) ===")
                print(ast_json)
            if save_ast:
                out_path = Path(save_ast)
                out_path.write_text(ast_json, encoding="utf-8")
                print(f"  AST saved to {out_path}")

        if compile_only:
            # In a real compiler, we'd emit code to 'output' here if provided
            if output:
                print(f"  (compile-only) No code emission implemented yet. "
                      f"Ignoring output='{output}'.")
            return

        # For now, just interpret (real compiler would generate machine code)
        interpreter = TongInterpreter()
        print("  Executing...")
        interpreter.interpret(program)
        print(f"Successfully executed {filename}")

    except FileNotFoundError:
        print(f"Error: File '{filename}' not found")
        sys.exit(1)
    except (ParseError, SyntaxError, ValueError, RuntimeError, OSError) as e:
        print(f"Error compiling {filename}: {e}")
        sys.exit(1)

def run_repl() -> None:
    """Run the interactive REPL"""
    repl = TongREPL()
    repl.run()

def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(
        description="TONG - The Ultimate Programming Language",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  tong                     # Start interactive REPL
  tong program.tong        # Compile and run program
  tong -c program.tong     # Compile to bytecode
  tong --help              # Show this help

TONG Language Features:
  - Zero-cost abstractions
  - Automatic parallelization
  - Memory safety without GC
  - Heterogeneous computing (CPU/GPU/NPU/FPGA)
  - WebAssembly compilation
  - Hot-swappable REPL
        """
    )

    parser.add_argument('file', nargs='?', help='TONG source file to compile/run')
    parser.add_argument('-c', '--compile', action='store_true',
                       help='Compile only (don\'t execute)')
    parser.add_argument('-o', '--output', help='Output file name')
    parser.add_argument('--print-ast', action='store_true',
                        help='Print the parsed AST as JSON and exit')
    parser.add_argument('--save-ast', metavar='FILE',
                        help='Save the parsed AST as JSON to FILE')
    parser.add_argument('--version', action='version', version='TONG 1.0.0')

    args = parser.parse_args()

    print("TONG Programming Language v1.0.0")
    print("The Ultimate Language for Heterogeneous Computing")
    print()

    if args.file:
        compile_file(
            filename=args.file,
            output=args.output,
            print_ast=args.print_ast,
            save_ast=args.save_ast,
            compile_only=args.compile,
        )
    else:
        run_repl()

if __name__ == "__main__":
    main()
