"""
TONG Language REPL (Read-Eval-Print Loop)
Interactive programming environment with hot compilation
"""

import sys
import traceback
from typing import Optional, List
import readline  # For command history and editing

from src.lexer import TongLexer, Token, TokenType
from src.parser import TongParser, ParseError
from src.interpreter import TongInterpreter, RuntimeError, Environment
from src.ast_nodes import *

class TongREPL:
    """Interactive REPL for TONG programming language"""
    
    def __init__(self):
        self.interpreter = TongInterpreter()
        self.history = []
        self.multiline_buffer = []
        self.in_multiline = False
        
        # Setup readline for better input handling
        try:
            readline.parse_and_bind('tab: complete')
            readline.parse_and_bind('set editing-mode emacs')
        except:
            pass  # readline not available on all systems
    
    def run(self):
        """Start the REPL"""
        print("TONG Language REPL v1.0")
        print("The ultimate programming language for heterogeneous computing")
        print("Type 'help' for commands, 'exit' to quit")
        print()
        
        while True:
            try:
                if self.in_multiline:
                    prompt = "... "
                else:
                    prompt = ">>> "
                
                line = input(prompt).strip()
                
                # Handle special commands
                if line in ['exit', 'quit']:
                    print("Goodbye!")
                    break
                elif line == 'help':
                    self.show_help()
                    continue
                elif line == 'clear':
                    self.clear_environment()
                    continue
                elif line == 'vars':
                    self.show_variables()
                    continue
                elif line == 'history':
                    self.show_history()
                    continue
                elif line == '':
                    if self.in_multiline:
                        # Empty line ends multiline input
                        self.execute_multiline()
                    continue
                
                # Check for multiline constructs
                if self.needs_multiline(line):
                    self.multiline_buffer.append(line)
                    self.in_multiline = True
                    continue
                
                # Single line execution
                if not self.in_multiline:
                    self.execute_line(line)
                else:
                    self.multiline_buffer.append(line)
                    if self.is_multiline_complete(self.multiline_buffer):
                        self.execute_multiline()
                
            except KeyboardInterrupt:
                print("\nKeyboardInterrupt")
                self.multiline_buffer.clear()
                self.in_multiline = False
            except EOFError:
                print("\nGoodbye!")
                break
            except Exception as e:
                print(f"Error: {e}")
                self.multiline_buffer.clear()
                self.in_multiline = False
    
    def needs_multiline(self, line: str) -> bool:
        """Check if line needs multiline input"""
        # Function definitions, control structures, etc.
        keywords = ['fn', 'if', 'while', 'for', 'match', 'parallel', 'distributed']
        for keyword in keywords:
            if line.startswith(keyword) and '{' in line and not line.rstrip().endswith('}'):
                return True
        
        # Unclosed braces
        open_braces = line.count('{') - line.count('}')
        if open_braces > 0:
            return True
        
        return False
    
    def is_multiline_complete(self, lines: List[str]) -> bool:
        """Check if multiline input is complete"""
        text = '\n'.join(lines)
        open_braces = text.count('{') - text.count('}')
        return open_braces <= 0
    
    def execute_line(self, line: str) -> None:
        """Execute a single line of TONG code"""
        try:
            # Add to history
            self.history.append(line)
            
            # Tokenize
            lexer = TongLexer(line)
            tokens = lexer.tokenize()
            
            # Parse
            parser = TongParser(tokens)
            
            # Handle expressions vs statements
            if self.is_expression(tokens):
                # Parse as expression and wrap in expression statement
                expr = parser.parse_expression()
                stmt = ExpressionStatement(expr)
                result = self.interpreter.execute_statement(stmt, self.interpreter.global_env)
                if result is not None and not isinstance(result, type(None)):
                    print(f"=> {result.value}")
            else:
                # Parse as statement
                program = parser.parse()
                self.interpreter.interpret(program)
        
        except ParseError as e:
            print(f"Parse error: {e}")
        except RuntimeError as e:
            print(f"Runtime error: {e}")
        except Exception as e:
            print(f"Unexpected error: {e}")
            traceback.print_exc()
    
    def execute_multiline(self) -> None:
        """Execute multiline code"""
        try:
            code = '\n'.join(self.multiline_buffer)
            self.history.append(code)
            
            # Tokenize
            lexer = TongLexer(code)
            tokens = lexer.tokenize()
            
            # Parse
            parser = TongParser(tokens)
            program = parser.parse()
            
            # Execute
            self.interpreter.interpret(program)
            
        except ParseError as e:
            print(f"Parse error: {e}")
        except RuntimeError as e:
            print(f"Runtime error: {e}")
        except Exception as e:
            print(f"Unexpected error: {e}")
            traceback.print_exc()
        finally:
            self.multiline_buffer.clear()
            self.in_multiline = False
    
    def is_expression(self, tokens: List[Token]) -> bool:
        """Check if tokens represent an expression vs statement"""
        if not tokens or tokens[0].type == TokenType.EOF:
            return False
        
        # Statement keywords
        statement_keywords = {
            TokenType.LET, TokenType.VAR, TokenType.FN, TokenType.IF, 
            TokenType.WHILE, TokenType.FOR, TokenType.MATCH, TokenType.RETURN,
            TokenType.BREAK, TokenType.CONTINUE
        }
        
        first_token = tokens[0]
        if first_token.type in statement_keywords:
            return False
        
        # Check for assignment (identifier = ...)
        if (len(tokens) >= 3 and 
            first_token.type == TokenType.IDENTIFIER and 
            tokens[1].type == TokenType.ASSIGN):
            return False
        
        return True
    
    def show_help(self) -> None:
        """Show REPL help"""
        print("""
TONG REPL Commands:
  help     - Show this help message
  exit     - Exit the REPL  
  quit     - Exit the REPL
  clear    - Clear all variables and functions
  vars     - Show all defined variables
  history  - Show command history

Language Features:
  Variables:     let x = 42, var y = 3.14
  Functions:     fn add(a, b) { a + b }
  Parallel:      parallel { ... }
  GPU Kernels:   gpu_kernel fn compute() { ... }
  Arrays:        [1, 2, 3, 4, 5]
  Control:       if, while, for, match

Built-in Functions:
  print(...)     - Print values
  len(array)     - Get array/string length
  sum(array)     - Sum array elements (auto-parallel)
  map(arr, fn)   - Map function over array (auto-parallel)

Examples:
  >>> let numbers = [1, 2, 3, 4, 5]
  >>> sum(numbers)
  => 15
  
  >>> fn square(x) { x * x }
  >>> map(numbers, square)
  => [1, 4, 9, 16, 25]
        """)
    
    def clear_environment(self) -> None:
        """Clear the environment"""
        self.interpreter = TongInterpreter()
        print("Environment cleared.")
    
    def show_variables(self) -> None:
        """Show all defined variables"""
        env = self.interpreter.global_env
        if not env.bindings:
            print("No variables defined.")
            return
        
        print("Defined variables:")
        for name, value in env.bindings.items():
            if not name.startswith('_'):  # Hide built-ins
                print(f"  {name}: {value.type_name} = {value.value}")
    
    def show_history(self) -> None:
        """Show command history"""
        if not self.history:
            print("No history.")
            return
        
        print("Command history:")
        for i, cmd in enumerate(self.history[-20:], 1):  # Show last 20
            print(f"  {i:2d}: {cmd}")

def main():
    """Main entry point for REPL"""
    repl = TongREPL()
    repl.run()

if __name__ == "__main__":
    main()