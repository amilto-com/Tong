#!/usr/bin/env python3
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'src'))

from src.lexer import TongLexer, TokenType

code = """fn main() {
    print("Hello")
}
main()"""

lexer = TongLexer(code)
tokens = lexer.tokenize()

for token in tokens:
    print(f"{token.type.name:15} | {repr(token.value):10} | Line {token.line}, Col {token.column}")