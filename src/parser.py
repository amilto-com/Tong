"""
TONG Language Parser
Parses tokens into an Abstract Syntax Tree (AST)
"""

from typing import List, Optional, Union
from src.lexer import Token, TokenType, TongLexer
from src.ast_nodes import *

class ParseError(Exception):
    """Exception raised for parsing errors"""
    def __init__(self, message: str, token: Token):
        self.message = message
        self.token = token
        super().__init__(f"Parse error at line {token.line}, column {token.column}: {message}")

class TongParser:
    """Parser for TONG programming language"""
    
    def __init__(self, tokens: List[Token]):
        self.tokens = tokens
        self.pos = 0
        self.current_token = tokens[0] if tokens else None
    
    def advance(self) -> None:
        """Move to the next token"""
        if self.pos < len(self.tokens) - 1:
            self.pos += 1
            self.current_token = self.tokens[self.pos]
    
    def peek(self, offset: int = 1) -> Optional[Token]:
        """Look ahead at a token"""
        peek_pos = self.pos + offset
        return self.tokens[peek_pos] if peek_pos < len(self.tokens) else None
    
    def match(self, *token_types: TokenType) -> bool:
        """Check if current token matches any of the given types"""
        return self.current_token and self.current_token.type in token_types
    
    def consume(self, token_type: TokenType, message: str = "") -> Token:
        """Consume a token of the expected type or raise error"""
        if not self.current_token or self.current_token.type != token_type:
            error_msg = message or f"Expected {token_type}, got {self.current_token.type if self.current_token else 'EOF'}"
            raise ParseError(error_msg, self.current_token or Token(TokenType.EOF, "", 0, 0))
        
        token = self.current_token
        self.advance()
        return token
    
    def skip_newlines(self) -> None:
        """Skip newline and comment tokens"""
        while self.match(TokenType.NEWLINE, TokenType.COMMENT):
            self.advance()
    
    def parse(self) -> Program:
        """Parse the token stream into a program"""
        statements = []
        
        while self.current_token and self.current_token.type != TokenType.EOF:
            self.skip_newlines()
            if self.current_token and self.current_token.type != TokenType.EOF:
                try:
                    stmt = self.parse_statement()
                    if stmt:
                        statements.append(stmt)
                except ParseError as e:
                    print(f"Parse error: {e}")
                    print(f"Current token: {self.current_token}")
                    raise
        
        return Program(statements)
    
    def parse_statement(self) -> Optional[Statement]:
        """Parse a statement"""
        self.skip_newlines()
        
        if not self.current_token or self.current_token.type == TokenType.EOF:
            return None
        
        # Skip right brace - it's handled by the caller
        if self.match(TokenType.RIGHT_BRACE):
            return None
        
        # Function declaration
        if self.match(TokenType.FN, TokenType.ASYNC, TokenType.GPU_KERNEL, TokenType.DISTRIBUTED):
            return self.parse_function_declaration()
        
        # Variable declaration
        elif self.match(TokenType.LET, TokenType.VAR):
            return self.parse_variable_declaration()
        
        # Control flow
        elif self.match(TokenType.IF):
            return self.parse_if_statement()
        elif self.match(TokenType.WHILE):
            return self.parse_while_statement()
        elif self.match(TokenType.FOR):
            return self.parse_for_statement()
        elif self.match(TokenType.MATCH):
            return self.parse_match_statement()
        
        # Flow control
        elif self.match(TokenType.RETURN):
            return self.parse_return_statement()
        elif self.match(TokenType.BREAK):
            self.advance()
            return BreakStatement()
        elif self.match(TokenType.CONTINUE):
            self.advance()
            return ContinueStatement()
        
        # Expression statement
        else:
            expr = self.parse_expression()
            return ExpressionStatement(expr)
    
    def parse_function_declaration(self) -> FunctionDeclaration:
        """Parse function declaration"""
        is_async = False
        is_gpu_kernel = False
        is_distributed = False
        
        # Handle function modifiers
        if self.match(TokenType.ASYNC):
            is_async = True
            self.advance()
        elif self.match(TokenType.GPU_KERNEL):
            is_gpu_kernel = True
            self.advance()
        elif self.match(TokenType.DISTRIBUTED):
            is_distributed = True
            self.advance()
        
        self.consume(TokenType.FN, "Expected 'fn'")
        name_token = self.consume(TokenType.IDENTIFIER, "Expected function name")
        
        # Generic parameters (optional)
        generic_params = None
        if self.match(TokenType.LESS_THAN):
            generic_params = self.parse_generic_parameters()
        
        # Parameters
        self.consume(TokenType.LEFT_PAREN, "Expected '('")
        parameters = self.parse_parameter_list()
        self.consume(TokenType.RIGHT_PAREN, "Expected ')'")
        
        # Return type (optional)
        return_type = None
        if self.match(TokenType.ARROW):
            self.advance()
            return_type = self.parse_type()
        
        # Function body
        self.consume(TokenType.LEFT_BRACE, "Expected '{'")
        body = []
        while not self.match(TokenType.RIGHT_BRACE) and self.current_token.type != TokenType.EOF:
            stmt = self.parse_statement()
            if stmt:
                body.append(stmt)
        self.consume(TokenType.RIGHT_BRACE, "Expected '}'")
        
        return FunctionDeclaration(
            name=name_token.value,
            parameters=parameters,
            return_type=return_type,
            body=body,
            is_async=is_async,
            is_gpu_kernel=is_gpu_kernel,
            is_distributed=is_distributed,
            generic_params=generic_params
        )
    
    def parse_parameter_list(self) -> List[Parameter]:
        """Parse function parameter list"""
        parameters = []
        
        while not self.match(TokenType.RIGHT_PAREN) and self.current_token.type != TokenType.EOF:
            if parameters:
                self.consume(TokenType.COMMA, "Expected ','")
            
            name_token = self.consume(TokenType.IDENTIFIER, "Expected parameter name")
            
            # Type annotation
            type_annotation = None
            if self.match(TokenType.COLON):
                self.advance()
                type_annotation = self.parse_type()
            
            # Default value
            default_value = None
            if self.match(TokenType.ASSIGN):
                self.advance()
                default_value = self.parse_expression()
            
            parameters.append(Parameter(
                name=name_token.value,
                type_annotation=type_annotation,
                default_value=default_value
            ))
        
        return parameters
    
    def parse_variable_declaration(self) -> VariableDeclaration:
        """Parse variable declaration"""
        is_mutable = self.match(TokenType.VAR)
        self.advance()  # consume 'let' or 'var'
        
        name_token = self.consume(TokenType.IDENTIFIER, "Expected variable name")
        
        # Type annotation
        type_annotation = None
        if self.match(TokenType.COLON):
            self.advance()
            type_annotation = self.parse_type()
        
        # Initializer
        initializer = None
        if self.match(TokenType.ASSIGN):
            self.advance()
            initializer = self.parse_expression()
        
        return VariableDeclaration(
            name=name_token.value,
            type_annotation=type_annotation,
            initializer=initializer,
            is_mutable=is_mutable
        )
    
    def parse_type(self) -> Type:
        """Parse type annotation"""
        if self.match(TokenType.IDENTIFIER):
            name = self.current_token.value
            self.advance()
            
            # Generic type
            if self.match(TokenType.LESS_THAN):
                self.advance()
                type_params = []
                while not self.match(TokenType.GREATER_THAN):
                    if type_params:
                        self.consume(TokenType.COMMA)
                    type_params.append(self.parse_type())
                self.consume(TokenType.GREATER_THAN)
                return GenericType(name, type_params)
            
            return PrimitiveType(name)
        
        elif self.match(TokenType.LEFT_BRACKET):
            # Array type
            self.advance()
            element_type = self.parse_type()
            
            # Optional size
            size = None
            if self.match(TokenType.SEMICOLON):
                self.advance()
                size = self.parse_expression()
            
            self.consume(TokenType.RIGHT_BRACKET)
            return ArrayType(element_type, size)
        
        elif self.match(TokenType.LEFT_PAREN):
            # Tuple type or function type
            self.advance()
            types = []
            while not self.match(TokenType.RIGHT_PAREN):
                if types:
                    self.consume(TokenType.COMMA)
                types.append(self.parse_type())
            self.consume(TokenType.RIGHT_PAREN)
            
            # Check if it's a function type
            if self.match(TokenType.ARROW):
                self.advance()
                return_type = self.parse_type()
                return FunctionType(types, return_type)
            
            return TupleType(types)
        
        elif self.match(TokenType.AMPERSAND):
            # Reference type
            self.advance()
            is_mutable = False
            if self.match(TokenType.VAR):
                is_mutable = True
                self.advance()
            target_type = self.parse_type()
            return ReferenceType(target_type, is_mutable)
        
        else:
            raise ParseError("Expected type", self.current_token)
    
    def parse_expression(self) -> Expression:
        """Parse expression with operator precedence"""
        return self.parse_logical_or()
    
    def parse_logical_or(self) -> Expression:
        """Parse logical OR expression"""
        expr = self.parse_logical_and()
        
        while self.match(TokenType.OR):
            op_token = self.current_token
            self.advance()
            right = self.parse_logical_and()
            expr = BinaryOperation(expr, BinaryOperator.OR, right)
        
        return expr
    
    def parse_logical_and(self) -> Expression:
        """Parse logical AND expression"""
        expr = self.parse_equality()
        
        while self.match(TokenType.AND):
            op_token = self.current_token
            self.advance()
            right = self.parse_equality()
            expr = BinaryOperation(expr, BinaryOperator.AND, right)
        
        return expr
    
    def parse_equality(self) -> Expression:
        """Parse equality expression"""
        expr = self.parse_comparison()
        
        while self.match(TokenType.EQUAL, TokenType.NOT_EQUAL):
            op_token = self.current_token
            self.advance()
            right = self.parse_comparison()
            
            op = BinaryOperator.EQUAL if op_token.type == TokenType.EQUAL else BinaryOperator.NOT_EQUAL
            expr = BinaryOperation(expr, op, right)
        
        return expr
    
    def parse_comparison(self) -> Expression:
        """Parse comparison expression"""
        expr = self.parse_addition()
        
        while self.match(TokenType.LESS_THAN, TokenType.LESS_EQUAL, 
                         TokenType.GREATER_THAN, TokenType.GREATER_EQUAL):
            op_token = self.current_token
            self.advance()
            right = self.parse_addition()
            
            op_map = {
                TokenType.LESS_THAN: BinaryOperator.LESS_THAN,
                TokenType.LESS_EQUAL: BinaryOperator.LESS_EQUAL,
                TokenType.GREATER_THAN: BinaryOperator.GREATER_THAN,
                TokenType.GREATER_EQUAL: BinaryOperator.GREATER_EQUAL,
            }
            expr = BinaryOperation(expr, op_map[op_token.type], right)
        
        return expr
    
    def parse_addition(self) -> Expression:
        """Parse addition/subtraction expression"""
        expr = self.parse_multiplication()
        
        while self.match(TokenType.PLUS, TokenType.MINUS):
            op_token = self.current_token
            self.advance()
            right = self.parse_multiplication()
            
            op = BinaryOperator.ADD if op_token.type == TokenType.PLUS else BinaryOperator.SUBTRACT
            expr = BinaryOperation(expr, op, right)
        
        return expr
    
    def parse_multiplication(self) -> Expression:
        """Parse multiplication/division expression"""
        expr = self.parse_unary()
        
        while self.match(TokenType.MULTIPLY, TokenType.DIVIDE, TokenType.MODULO):
            op_token = self.current_token
            self.advance()
            right = self.parse_unary()
            
            op_map = {
                TokenType.MULTIPLY: BinaryOperator.MULTIPLY,
                TokenType.DIVIDE: BinaryOperator.DIVIDE,
                TokenType.MODULO: BinaryOperator.MODULO,
            }
            expr = BinaryOperation(expr, op_map[op_token.type], right)
        
        return expr
    
    def parse_unary(self) -> Expression:
        """Parse unary expression"""
        if self.match(TokenType.MINUS, TokenType.NOT, TokenType.AMPERSAND, TokenType.MULTIPLY):
            op_token = self.current_token
            self.advance()
            operand = self.parse_unary()
            
            op_map = {
                TokenType.MINUS: UnaryOperator.NEGATE,
                TokenType.NOT: UnaryOperator.NOT,
                TokenType.AMPERSAND: UnaryOperator.REFERENCE,
                TokenType.MULTIPLY: UnaryOperator.DEREFERENCE,
            }
            return UnaryOperation(op_map[op_token.type], operand)
        
        return self.parse_postfix()
    
    def parse_postfix(self) -> Expression:
        """Parse postfix expressions (field access, method calls, indexing)"""
        expr = self.parse_primary()
        
        while True:
            if self.match(TokenType.DOT):
                self.advance()
                field_name = self.consume(TokenType.IDENTIFIER, "Expected field name").value
                
                # Check for method call
                if self.match(TokenType.LEFT_PAREN):
                    self.advance()
                    args = self.parse_argument_list()
                    self.consume(TokenType.RIGHT_PAREN)
                    expr = MethodCall(expr, field_name, args)
                else:
                    expr = FieldAccess(expr, field_name)
            
            elif self.match(TokenType.LEFT_BRACKET):
                self.advance()
                index = self.parse_expression()
                self.consume(TokenType.RIGHT_BRACKET)
                expr = IndexAccess(expr, index)
            
            elif self.match(TokenType.LEFT_PAREN):
                self.advance()
                args = self.parse_argument_list()
                self.consume(TokenType.RIGHT_PAREN)
                expr = FunctionCall(expr, args)
            
            else:
                break
        
        return expr
    
    def parse_primary(self) -> Expression:
        """Parse primary expressions"""
        # Literals
        if self.match(TokenType.INTEGER):
            value = int(self.current_token.value)
            self.advance()
            return IntegerLiteral(value)
        
        elif self.match(TokenType.FLOAT):
            value = float(self.current_token.value)
            self.advance()
            return FloatLiteral(value)
        
        elif self.match(TokenType.STRING):
            value = self.current_token.value
            self.advance()
            return StringLiteral(value)
        
        elif self.match(TokenType.TRUE, TokenType.FALSE):
            value = self.current_token.type == TokenType.TRUE
            self.advance()
            return BooleanLiteral(value)
        
        elif self.match(TokenType.NONE):
            self.advance()
            return NoneLiteral()
        
        # Identifier
        elif self.match(TokenType.IDENTIFIER):
            name = self.current_token.value
            self.advance()
            return Identifier(name)
        
        # Parenthesized expression
        elif self.match(TokenType.LEFT_PAREN):
            self.advance()
            expr = self.parse_expression()
            self.consume(TokenType.RIGHT_PAREN)
            return expr
        
        # Array literal
        elif self.match(TokenType.LEFT_BRACKET):
            self.advance()
            elements = []
            while not self.match(TokenType.RIGHT_BRACKET):
                if elements:
                    self.consume(TokenType.COMMA)
                elements.append(self.parse_expression())
            self.consume(TokenType.RIGHT_BRACKET)
            return ArrayLiteral(elements)
        
        # Lambda expression
        elif self.match(TokenType.PIPE):
            return self.parse_lambda()
        
        # Parallel block
        elif self.match(TokenType.PARALLEL):
            return self.parse_parallel_block()
        
        # Await expression
        elif self.match(TokenType.AWAIT):
            self.advance()
            expr = self.parse_expression()
            return AwaitExpression(expr)
        
        else:
            raise ParseError("Unexpected token", self.current_token)
    
    def parse_argument_list(self) -> List[Expression]:
        """Parse function argument list"""
        args = []
        while not self.match(TokenType.RIGHT_PAREN) and self.current_token.type != TokenType.EOF:
            if args:
                self.consume(TokenType.COMMA)
            args.append(self.parse_expression())
        return args
    
    def parse_lambda(self) -> Lambda:
        """Parse lambda expression"""
        self.consume(TokenType.PIPE)
        
        # Parameters
        parameters = []
        while not self.match(TokenType.PIPE):
            if parameters:
                self.consume(TokenType.COMMA)
            
            name_token = self.consume(TokenType.IDENTIFIER)
            type_annotation = None
            if self.match(TokenType.COLON):
                self.advance()
                type_annotation = self.parse_type()
            
            parameters.append(Parameter(name_token.value, type_annotation))
        
        self.consume(TokenType.PIPE)
        
        # Return type (optional)
        return_type = None
        if self.match(TokenType.ARROW):
            self.advance()
            return_type = self.parse_type()
        
        # Body
        body = self.parse_expression()
        
        return Lambda(parameters, body, return_type)
    
    def parse_parallel_block(self) -> ParallelBlock:
        """Parse parallel block"""
        self.consume(TokenType.PARALLEL)
        self.consume(TokenType.LEFT_BRACE)
        
        statements = []
        while not self.match(TokenType.RIGHT_BRACE) and self.current_token.type != TokenType.EOF:
            stmt = self.parse_statement()
            if stmt:
                statements.append(stmt)
        
        self.consume(TokenType.RIGHT_BRACE)
        return ParallelBlock(statements)
    
    def parse_if_statement(self) -> IfStatement:
        """Parse if statement"""
        self.consume(TokenType.IF)
        condition = self.parse_expression()
        
        self.consume(TokenType.LEFT_BRACE)
        then_body = []
        while not self.match(TokenType.RIGHT_BRACE):
            stmt = self.parse_statement()
            if stmt:
                then_body.append(stmt)
        self.consume(TokenType.RIGHT_BRACE)
        
        else_body = None
        if self.match(TokenType.ELSE):
            self.advance()
            self.consume(TokenType.LEFT_BRACE)
            else_body = []
            while not self.match(TokenType.RIGHT_BRACE):
                stmt = self.parse_statement()
                if stmt:
                    else_body.append(stmt)
            self.consume(TokenType.RIGHT_BRACE)
        
        return IfStatement(condition, then_body, else_body)
    
    def parse_while_statement(self) -> WhileLoop:
        """Parse while statement"""
        self.consume(TokenType.WHILE)
        condition = self.parse_expression()
        
        self.consume(TokenType.LEFT_BRACE)
        body = []
        while not self.match(TokenType.RIGHT_BRACE):
            stmt = self.parse_statement()
            if stmt:
                body.append(stmt)
        self.consume(TokenType.RIGHT_BRACE)
        
        return WhileLoop(condition, body)
    
    def parse_for_statement(self) -> ForLoop:
        """Parse for statement"""
        self.consume(TokenType.FOR)
        variable = self.consume(TokenType.IDENTIFIER).value
        # TODO: Add 'in' keyword to lexer
        # self.consume(TokenType.IN)
        iterable = self.parse_expression()
        
        self.consume(TokenType.LEFT_BRACE)
        body = []
        while not self.match(TokenType.RIGHT_BRACE):
            stmt = self.parse_statement()
            if stmt:
                body.append(stmt)
        self.consume(TokenType.RIGHT_BRACE)
        
        return ForLoop(variable, iterable, body)
    
    def parse_match_statement(self) -> MatchStatement:
        """Parse match statement"""
        self.consume(TokenType.MATCH)
        expr = self.parse_expression()
        
        self.consume(TokenType.LEFT_BRACE)
        arms = []
        while not self.match(TokenType.RIGHT_BRACE):
            # TODO: Implement pattern parsing
            pattern = IdentifierPattern("_")  # Placeholder
            
            if self.match(TokenType.IF):
                self.advance()
                guard = self.parse_expression()
            else:
                guard = None
            
            self.consume(TokenType.FAT_ARROW)
            body = self.parse_expression()
            
            arms.append(MatchArm(pattern, guard, body))
            
            if self.match(TokenType.COMMA):
                self.advance()
        
        self.consume(TokenType.RIGHT_BRACE)
        return MatchStatement(expr, arms)
    
    def parse_return_statement(self) -> ReturnStatement:
        """Parse return statement"""
        self.consume(TokenType.RETURN)
        
        value = None
        if not self.match(TokenType.NEWLINE, TokenType.SEMICOLON, TokenType.RIGHT_BRACE):
            value = self.parse_expression()
        
        return ReturnStatement(value)
    
    def parse_generic_parameters(self) -> List[str]:
        """Parse generic type parameters"""
        self.consume(TokenType.LESS_THAN)
        params = []
        
        while not self.match(TokenType.GREATER_THAN):
            if params:
                self.consume(TokenType.COMMA)
            params.append(self.consume(TokenType.IDENTIFIER).value)
        
        self.consume(TokenType.GREATER_THAN)
        return params