"""
TONG Language Lexer
Tokenizes TONG source code into a stream of tokens
"""

import re
from enum import Enum, auto
from dataclasses import dataclass
from typing import List, Optional, Iterator

class TokenType(Enum):
    # Literals
    INTEGER = auto()
    FLOAT = auto()
    STRING = auto()
    CHAR = auto()
    BOOLEAN = auto()
    
    # Identifiers and keywords
    IDENTIFIER = auto()
    
    # Keywords
    LET = auto()
    VAR = auto()
    FN = auto()
    IF = auto()
    ELSE = auto()
    MATCH = auto()
    WHILE = auto()
    FOR = auto()
    LOOP = auto()
    BREAK = auto()
    CONTINUE = auto()
    RETURN = auto()
    ASYNC = auto()
    AWAIT = auto()
    PARALLEL = auto()
    DISTRIBUTED = auto()
    GPU_KERNEL = auto()
    OWNED = auto()
    SHARED = auto()
    TRUE = auto()
    FALSE = auto()
    NONE = auto()
    SOME = auto()
    
    # Operators
    PLUS = auto()
    MINUS = auto()
    MULTIPLY = auto()
    DIVIDE = auto()
    MODULO = auto()
    POWER = auto()
    ASSIGN = auto()
    PLUS_ASSIGN = auto()
    MINUS_ASSIGN = auto()
    MULTIPLY_ASSIGN = auto()
    DIVIDE_ASSIGN = auto()
    
    # Comparison
    EQUAL = auto()
    NOT_EQUAL = auto()
    LESS_THAN = auto()
    LESS_EQUAL = auto()
    GREATER_THAN = auto()
    GREATER_EQUAL = auto()
    
    # Logical
    AND = auto()
    OR = auto()
    NOT = auto()
    
    # Punctuation
    SEMICOLON = auto()
    COMMA = auto()
    DOT = auto()
    COLON = auto()
    DOUBLE_COLON = auto()
    QUESTION = auto()
    ARROW = auto()
    FAT_ARROW = auto()
    PIPE = auto()
    AMPERSAND = auto()
    
    # Brackets
    LEFT_PAREN = auto()
    RIGHT_PAREN = auto()
    LEFT_BRACE = auto()
    RIGHT_BRACE = auto()
    LEFT_BRACKET = auto()
    RIGHT_BRACKET = auto()
    
    # Special
    NEWLINE = auto()
    EOF = auto()
    COMMENT = auto()

@dataclass
class Token:
    type: TokenType
    value: str
    line: int
    column: int

class TongLexer:
    """Lexical analyzer for TONG programming language"""
    
    KEYWORDS = {
        'let': TokenType.LET,
        'var': TokenType.VAR,
        'fn': TokenType.FN,
        'if': TokenType.IF,
        'else': TokenType.ELSE,
        'match': TokenType.MATCH,
        'while': TokenType.WHILE,
        'for': TokenType.FOR,
        'loop': TokenType.LOOP,
        'break': TokenType.BREAK,
        'continue': TokenType.CONTINUE,
        'return': TokenType.RETURN,
        'async': TokenType.ASYNC,
        'await': TokenType.AWAIT,
        'parallel': TokenType.PARALLEL,
        'distributed': TokenType.DISTRIBUTED,
        'gpu_kernel': TokenType.GPU_KERNEL,
        'owned': TokenType.OWNED,
        'shared': TokenType.SHARED,
        'true': TokenType.TRUE,
        'false': TokenType.FALSE,
        'None': TokenType.NONE,
        'Some': TokenType.SOME,
    }
    
    def __init__(self, source: str):
        self.source = source
        self.pos = 0
        self.line = 1
        self.column = 1
        self.tokens = []
    
    def current_char(self) -> Optional[str]:
        """Get current character or None if at end"""
        return self.source[self.pos] if self.pos < len(self.source) else None
    
    def peek_char(self, offset: int = 1) -> Optional[str]:
        """Peek ahead at character"""
        peek_pos = self.pos + offset
        return self.source[peek_pos] if peek_pos < len(self.source) else None
    
    def advance(self) -> None:
        """Move to next character"""
        if self.pos < len(self.source) and self.source[self.pos] == '\n':
            self.line += 1
            self.column = 1
        else:
            self.column += 1
        self.pos += 1
    
    def skip_whitespace(self) -> None:
        """Skip whitespace except newlines"""
        while self.current_char() and self.current_char() in ' \t\r':
            self.advance()
    
    def read_number(self) -> Token:
        """Read integer or float literal"""
        start_pos = self.pos
        start_column = self.column
        
        # Read digits
        while self.current_char() and self.current_char().isdigit():
            self.advance()
        
        # Check for decimal point
        if self.current_char() == '.' and self.peek_char() and self.peek_char().isdigit():
            self.advance()  # consume '.'
            while self.current_char() and self.current_char().isdigit():
                self.advance()
            return Token(TokenType.FLOAT, self.source[start_pos:self.pos], self.line, start_column)
        
        return Token(TokenType.INTEGER, self.source[start_pos:self.pos], self.line, start_column)
    
    def read_string(self) -> Token:
        """Read string literal"""
        start_column = self.column
        quote_char = self.current_char()
        self.advance()  # consume opening quote
        
        value = ""
        while self.current_char() and self.current_char() != quote_char:
            if self.current_char() == '\\':
                self.advance()
                escape_char = self.current_char()
                if escape_char == 'n':
                    value += '\n'
                elif escape_char == 't':
                    value += '\t'
                elif escape_char == 'r':
                    value += '\r'
                elif escape_char == '\\':
                    value += '\\'
                elif escape_char == quote_char:
                    value += quote_char
                else:
                    value += escape_char or ''
                if escape_char:
                    self.advance()
            else:
                value += self.current_char()
                self.advance()
        
        if self.current_char() == quote_char:
            self.advance()  # consume closing quote
        
        return Token(TokenType.STRING, value, self.line, start_column)
    
    def read_identifier(self) -> Token:
        """Read identifier or keyword"""
        start_pos = self.pos
        start_column = self.column
        
        while (self.current_char() and 
               (self.current_char().isalnum() or self.current_char() in '_')):
            self.advance()
        
        value = self.source[start_pos:self.pos]
        token_type = self.KEYWORDS.get(value, TokenType.IDENTIFIER)
        
        return Token(token_type, value, self.line, start_column)
    
    def tokenize(self) -> List[Token]:
        """Tokenize the entire source code"""
        while self.pos < len(self.source):
            self.skip_whitespace()
            
            char = self.current_char()
            if not char:
                break
            
            # Numbers
            if char.isdigit():
                self.tokens.append(self.read_number())
            
            # Strings
            elif char in '"\'':
                self.tokens.append(self.read_string())
            
            # Identifiers and keywords
            elif char.isalpha() or char == '_':
                self.tokens.append(self.read_identifier())
            
            # Comments
            elif char == '/' and self.peek_char() == '/':
                # Line comment - skip to end of line
                while self.current_char() and self.current_char() != '\n':
                    self.advance()
                continue
            elif char == '/' and self.peek_char() == '*':
                # Block comment - skip to */
                self.advance()  # consume '/'
                self.advance()  # consume '*'
                while self.current_char():
                    if self.current_char() == '*' and self.peek_char() == '/':
                        self.advance()  # consume '*'
                        self.advance()  # consume '/'
                        break
                    self.advance()
                continue
            
            # Two-character operators
            elif char == '=' and self.peek_char() == '=':
                self.tokens.append(Token(TokenType.EQUAL, '==', self.line, self.column))
                self.advance()
                self.advance()
            elif char == '!' and self.peek_char() == '=':
                self.tokens.append(Token(TokenType.NOT_EQUAL, '!=', self.line, self.column))
                self.advance()
                self.advance()
            elif char == '<' and self.peek_char() == '=':
                self.tokens.append(Token(TokenType.LESS_EQUAL, '<=', self.line, self.column))
                self.advance()
                self.advance()
            elif char == '>' and self.peek_char() == '=':
                self.tokens.append(Token(TokenType.GREATER_EQUAL, '>=', self.line, self.column))
                self.advance()
                self.advance()
            elif char == '-' and self.peek_char() == '>':
                self.tokens.append(Token(TokenType.ARROW, '->', self.line, self.column))
                self.advance()
                self.advance()
            elif char == '=' and self.peek_char() == '>':
                self.tokens.append(Token(TokenType.FAT_ARROW, '=>', self.line, self.column))
                self.advance()
                self.advance()
            elif char == ':' and self.peek_char() == ':':
                self.tokens.append(Token(TokenType.DOUBLE_COLON, '::', self.line, self.column))
                self.advance()
                self.advance()
            
            # Single-character tokens
            elif char == '+':
                self.tokens.append(Token(TokenType.PLUS, char, self.line, self.column))
                self.advance()
            elif char == '-':
                self.tokens.append(Token(TokenType.MINUS, char, self.line, self.column))
                self.advance()
            elif char == '*':
                self.tokens.append(Token(TokenType.MULTIPLY, char, self.line, self.column))
                self.advance()
            elif char == '/':
                self.tokens.append(Token(TokenType.DIVIDE, char, self.line, self.column))
                self.advance()
            elif char == '%':
                self.tokens.append(Token(TokenType.MODULO, char, self.line, self.column))
                self.advance()
            elif char == '=':
                self.tokens.append(Token(TokenType.ASSIGN, char, self.line, self.column))
                self.advance()
            elif char == '<':
                self.tokens.append(Token(TokenType.LESS_THAN, char, self.line, self.column))
                self.advance()
            elif char == '>':
                self.tokens.append(Token(TokenType.GREATER_THAN, char, self.line, self.column))
                self.advance()
            elif char == '!':
                self.tokens.append(Token(TokenType.NOT, char, self.line, self.column))
                self.advance()
            elif char == ';':
                self.tokens.append(Token(TokenType.SEMICOLON, char, self.line, self.column))
                self.advance()
            elif char == ',':
                self.tokens.append(Token(TokenType.COMMA, char, self.line, self.column))
                self.advance()
            elif char == '.':
                self.tokens.append(Token(TokenType.DOT, char, self.line, self.column))
                self.advance()
            elif char == ':':
                self.tokens.append(Token(TokenType.COLON, char, self.line, self.column))
                self.advance()
            elif char == '?':
                self.tokens.append(Token(TokenType.QUESTION, char, self.line, self.column))
                self.advance()
            elif char == '|':
                self.tokens.append(Token(TokenType.PIPE, char, self.line, self.column))
                self.advance()
            elif char == '&':
                self.tokens.append(Token(TokenType.AMPERSAND, char, self.line, self.column))
                self.advance()
            elif char == '(':
                self.tokens.append(Token(TokenType.LEFT_PAREN, char, self.line, self.column))
                self.advance()
            elif char == ')':
                self.tokens.append(Token(TokenType.RIGHT_PAREN, char, self.line, self.column))
                self.advance()
            elif char == '{':
                self.tokens.append(Token(TokenType.LEFT_BRACE, char, self.line, self.column))
                self.advance()
            elif char == '}':
                self.tokens.append(Token(TokenType.RIGHT_BRACE, char, self.line, self.column))
                self.advance()
            elif char == '[':
                self.tokens.append(Token(TokenType.LEFT_BRACKET, char, self.line, self.column))
                self.advance()
            elif char == ']':
                self.tokens.append(Token(TokenType.RIGHT_BRACKET, char, self.line, self.column))
                self.advance()
            elif char == '\n':
                self.tokens.append(Token(TokenType.NEWLINE, char, self.line, self.column))
                self.advance()
            else:
                # Unknown character, skip
                self.advance()
        
        self.tokens.append(Token(TokenType.EOF, '', self.line, self.column))
        return self.tokens