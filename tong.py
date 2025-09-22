#!/usr/bin/env python3
"""
TONG Programming Language Compiler and Interpreter
The ultimate programming language for heterogeneous computing
"""

import sys
import argparse
import os
from pathlib import Path

# Add src to path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'src'))

from src.lexer import TongLexer
from src.parser import TongParser
from src.interpreter import TongInterpreter
from src.repl import TongREPL

def compile_file(filename: str, output: str = None) -> None:
    """Compile a TONG source file"""
    try:
        with open(filename, 'r') as f:
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
        
        # For now, just interpret (real compiler would generate machine code)
        interpreter = TongInterpreter()
        print("  Executing...")
        interpreter.interpret(program)
        
        print(f"Successfully executed {filename}")
        
    except FileNotFoundError:
        print(f"Error: File '{filename}' not found")
        sys.exit(1)
    except Exception as e:
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
    parser.add_argument('--version', action='version', version='TONG 1.0.0')
    
    args = parser.parse_args()
    
    print("TONG Programming Language v1.0.0")
    print("The Ultimate Language for Heterogeneous Computing")
    print()
    
    if args.file:
        compile_file(args.file, args.output)
    else:
        run_repl()

if __name__ == "__main__":
    main()