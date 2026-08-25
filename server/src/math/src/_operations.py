# SPDX-License-Identifier: AGPL-3.0-or-later

"""Private implementation of the math module's public operations."""

from .models import Calculation, Operation


def add(left: float, right: float) -> Calculation:
    """Add two values."""
    left, right = _normalize_operands(left, right)
    return _create_calculation(Operation.ADD, left, right, left + right)


def subtract(left: float, right: float) -> Calculation:
    """Subtract the right value from the left value."""
    left, right = _normalize_operands(left, right)
    return _create_calculation(Operation.SUBTRACT, left, right, left - right)


def multiply(left: float, right: float) -> Calculation:
    """Multiply two values."""
    left, right = _normalize_operands(left, right)
    return _create_calculation(Operation.MULTIPLY, left, right, left * right)


def divide(left: float, right: float) -> Calculation:
    """Divide the left value by the right value.

    Raises:
        ValueError: If ``right`` is zero.
    """
    left, right = _normalize_operands(left, right)
    _require_nonzero_divisor(right)
    return _create_calculation(Operation.DIVIDE, left, right, left / right)


def _normalize_operands(left: float, right: float) -> tuple[float, float]:
    """Convert operands to the public interface's runtime representation."""
    return float(left), float(right)


def _create_calculation(
    operation: Operation, left: float, right: float, result: float
) -> Calculation:
    """Build the shared public result type for an operation."""
    return Calculation(operation=operation, left=left, right=right, result=result)


def _require_nonzero_divisor(divisor: float) -> None:
    """Reject an invalid divisor before division is performed."""
    if divisor == 0:
        raise ValueError("Cannot divide by zero.")
