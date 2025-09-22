from dataclasses import is_dataclass, fields
from enum import Enum
from typing import Any


def ast_to_dict(node: Any) -> Any:
    """Recursively convert an AST node (dataclass-based) into a JSON-serializable dict.

    - Adds a "type" field with the node class name for readability
    - Converts Enums to their value
    - Handles lists and primitives
    """
    if node is None:
        return None

    if isinstance(node, (int, float, str, bool)):
        return node

    if isinstance(node, Enum):
        # Use the enum's value for compactness
        return node.value

    if isinstance(node, list):
        return [ast_to_dict(elem) for elem in node]

    if is_dataclass(node):
        data = {"type": getattr(type(node), "__name__", "ASTNode")}
        for f in fields(node):
            val = getattr(node, f.name)
            data[f.name] = ast_to_dict(val)
        return data

    # Fallback: string representation
    return repr(node)
