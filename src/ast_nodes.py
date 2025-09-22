"""
TONG Language Abstract Syntax Tree (AST) Definitions
"""

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import List, Optional, Union, Any
from enum import Enum

class ASTNode(ABC):
    """Base class for all AST nodes"""
    pass

class Expression(ASTNode):
    """Base class for expressions"""
    pass

class Statement(ASTNode):
    """Base class for statements"""
    pass

class Type(ASTNode):
    """Base class for type annotations"""
    pass

# Type System
@dataclass
class PrimitiveType(Type):
    name: str  # i32, f64, bool, String, etc.

@dataclass
class ArrayType(Type):
    element_type: Type
    size: Optional[Expression] = None

@dataclass
class TupleType(Type):
    element_types: List[Type]

@dataclass
class FunctionType(Type):
    param_types: List[Type]
    return_type: Type
    is_async: bool = False

@dataclass
class GenericType(Type):
    name: str
    type_params: List[Type]

@dataclass
class ReferenceType(Type):
    target_type: Type
    is_mutable: bool = False

# Literals
@dataclass
class IntegerLiteral(Expression):
    value: int

@dataclass
class FloatLiteral(Expression):
    value: float

@dataclass
class StringLiteral(Expression):
    value: str

@dataclass
class BooleanLiteral(Expression):
    value: bool

@dataclass
class NoneLiteral(Expression):
    pass

@dataclass
class ArrayLiteral(Expression):
    elements: List[Expression]

@dataclass
class TupleLiteral(Expression):
    elements: List[Expression]

# Identifiers and Variables
@dataclass
class Identifier(Expression):
    name: str

@dataclass
class FieldAccess(Expression):
    object: Expression
    field: str

@dataclass
class IndexAccess(Expression):
    object: Expression
    index: Expression

@dataclass
class MethodCall(Expression):
    object: Expression
    method: str
    arguments: List[Expression]

# Binary Operations
class BinaryOperator(Enum):
    ADD = "+"
    SUBTRACT = "-"
    MULTIPLY = "*"
    DIVIDE = "/"
    MODULO = "%"
    POWER = "**"
    EQUAL = "=="
    NOT_EQUAL = "!="
    LESS_THAN = "<"
    LESS_EQUAL = "<="
    GREATER_THAN = ">"
    GREATER_EQUAL = ">="
    AND = "&&"
    OR = "||"

@dataclass
class BinaryOperation(Expression):
    left: Expression
    operator: BinaryOperator
    right: Expression

# Unary Operations
class UnaryOperator(Enum):
    NEGATE = "-"
    NOT = "!"
    REFERENCE = "&"
    DEREFERENCE = "*"

@dataclass
class UnaryOperation(Expression):
    operator: UnaryOperator
    operand: Expression

# Function Calls
@dataclass
class FunctionCall(Expression):
    function: Expression
    arguments: List[Expression]

@dataclass
class Lambda(Expression):
    parameters: List['Parameter']
    body: Expression
    return_type: Optional[Type] = None

# Control Flow Expressions
@dataclass
class IfExpression(Expression):
    condition: Expression
    then_expr: Expression
    else_expr: Optional[Expression] = None

@dataclass
class MatchExpression(Expression):
    expr: Expression
    arms: List['MatchArm']

@dataclass
class MatchArm:
    pattern: 'Pattern'
    guard: Optional[Expression]
    body: Expression

# Patterns
class Pattern(ASTNode):
    pass

@dataclass
class IdentifierPattern(Pattern):
    name: str

@dataclass
class LiteralPattern(Pattern):
    value: Expression

@dataclass
class TuplePattern(Pattern):
    patterns: List[Pattern]

@dataclass
class ArrayPattern(Pattern):
    patterns: List[Pattern]

@dataclass
class WildcardPattern(Pattern):
    pass

# Async/Await
@dataclass
class AwaitExpression(Expression):
    expr: Expression

@dataclass
class AsyncBlock(Expression):
    statements: List[Statement]

# Parallel Computing
@dataclass
class ParallelBlock(Expression):
    statements: List[Statement]

@dataclass
class DistributedBlock(Expression):
    statements: List[Statement]

@dataclass
class GpuKernelCall(Expression):
    function: Expression
    arguments: List[Expression]
    grid_size: Optional[Expression] = None
    block_size: Optional[Expression] = None

# Statements
@dataclass
class ExpressionStatement(Statement):
    expression: Expression

@dataclass
class VariableDeclaration(Statement):
    name: str
    type_annotation: Optional[Type]
    initializer: Optional[Expression]
    is_mutable: bool = False

@dataclass
class Assignment(Statement):
    target: Expression
    value: Expression

@dataclass
class Parameter:
    name: str
    type_annotation: Optional[Type]
    default_value: Optional[Expression] = None

@dataclass
class FunctionDeclaration(Statement):
    name: str
    parameters: List[Parameter]
    return_type: Optional[Type]
    body: List[Statement]
    is_async: bool = False
    is_gpu_kernel: bool = False
    is_distributed: bool = False
    generic_params: Optional[List[str]] = None

@dataclass
class ReturnStatement(Statement):
    value: Optional[Expression] = None

@dataclass
class BreakStatement(Statement):
    pass

@dataclass
class ContinueStatement(Statement):
    pass

# Control Flow Statements
@dataclass
class IfStatement(Statement):
    condition: Expression
    then_body: List[Statement]
    else_body: Optional[List[Statement]] = None

@dataclass
class WhileLoop(Statement):
    condition: Expression
    body: List[Statement]

@dataclass
class ForLoop(Statement):
    variable: str
    iterable: Expression
    body: List[Statement]

@dataclass
class MatchStatement(Statement):
    expr: Expression
    arms: List[MatchArm]

# Module and Program Structure
@dataclass
class ImportStatement(Statement):
    module_path: List[str]
    items: Optional[List[str]] = None  # None means import all

@dataclass
class ModuleDeclaration(Statement):
    name: str
    statements: List[Statement]

@dataclass
class Program(ASTNode):
    statements: List[Statement]