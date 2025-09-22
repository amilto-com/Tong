"""
Deprecated module: use the Rust implementation in rust/tong.
"""

raise RuntimeError("TONG Python AST retired; use the Rust CLI (see README.md)")
"""
TONG Language Abstract Syntax Tree (AST) Definitions
"""

from abc import ABC
from dataclasses import dataclass
from typing import List, Optional
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
    """Represents primitive types like i32, f64, bool, String."""
    name: str  # i32, f64, bool, String, etc.

@dataclass
class ArrayType(Type):
    """Represents array types with optional size specification."""
    element_type: Type
    size: Optional[Expression] = None

@dataclass
class TupleType(Type):
    """Represents tuple types with multiple element types."""
    element_types: List[Type]

@dataclass
class FunctionType(Type):
    """Represents function types with parameter and return types."""
    param_types: List[Type]
    return_type: Type
    is_async: bool = False

@dataclass
class GenericType(Type):
    """Represents generic types with type parameters."""
    name: str
    type_params: List[Type]

@dataclass
class ReferenceType(Type):
    """Represents reference types with mutability information."""
    target_type: Type
    is_mutable: bool = False

# Literals
@dataclass
class IntegerLiteral(Expression):
    """Represents integer literal expressions."""
    value: int

@dataclass
class FloatLiteral(Expression):
    """Represents floating-point literal expressions."""
    value: float

@dataclass
class StringLiteral(Expression):
    """Represents string literal expressions."""
    value: str

@dataclass
class BooleanLiteral(Expression):
    """Represents boolean literal expressions."""
    value: bool

@dataclass
class NoneLiteral(Expression):
    """Represents None/null literal expressions."""

@dataclass
class ArrayLiteral(Expression):
    """Represents array literal expressions."""
    elements: List[Expression]

@dataclass
class TupleLiteral(Expression):
    """Represents tuple literal expressions."""
    elements: List[Expression]

# Identifiers and Variables
@dataclass
class Identifier(Expression):
    """Represents identifier expressions (variable names)."""
    name: str

@dataclass
class FieldAccess(Expression):
    """Represents field access expressions (object.field)."""
    object: Expression
    field: str

@dataclass
class IndexAccess(Expression):
    """Represents array/object index access expressions (obj[index])."""
    object: Expression
    index: Expression

@dataclass
class MethodCall(Expression):
    """Represents method call expressions (object.method(args))."""
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
    """Represents binary operation expressions (left op right)."""
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
    """Represents unary operation expressions (op operand)."""
    operator: UnaryOperator
    operand: Expression

# Function Calls
@dataclass
class FunctionCall(Expression):
    """Represents function call expressions."""
    function: Expression
    arguments: List[Expression]

@dataclass
class Lambda(Expression):
    """Represents lambda function expressions."""
    parameters: List['Parameter']
    body: Expression
    return_type: Optional[Type] = None

# Control Flow Expressions
@dataclass
class IfExpression(Expression):
    """Represents conditional if expressions."""
    condition: Expression
    then_expr: Expression
    else_expr: Optional[Expression] = None

@dataclass
class MatchExpression(Expression):
    """Represents pattern matching expressions."""
    expr: Expression
    arms: List['MatchArm']

@dataclass
class MatchArm:
    """Represents a single arm in a match expression."""
    pattern: 'Pattern'
    guard: Optional[Expression]
    body: Expression

# Patterns
class Pattern(ASTNode):
    """Base class for all pattern expressions."""

@dataclass
class IdentifierPattern(Pattern):
    """Represents identifier patterns in pattern matching."""
    name: str

@dataclass
class LiteralPattern(Pattern):
    """Represents literal patterns in pattern matching."""
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
    alias: Optional[str] = None        # For 'import foo as bar'

@dataclass
class ModuleDeclaration(Statement):
    name: str
    statements: List[Statement]

@dataclass
class Program(ASTNode):
    statements: List[Statement]
