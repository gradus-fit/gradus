# SPDX-License-Identifier: AGPL-3.0-or-later

"""Tests for the math module's public interface."""

from collections.abc import Callable

import pytest

from .. import Calculation, Operation, add, divide, multiply, subtract

MODULE_NAME = __name__.rsplit(".tests.", maxsplit=1)[0]
OperationFunction = Callable[[float, float], Calculation]


@pytest.mark.public_api(
    MODULE_NAME,
    "Calculation",
    "Operation",
    "add",
    "divide",
    "multiply",
    "subtract",
)
@pytest.mark.parametrize(
    ("operation", "left", "right", "expected_result", "expected_operation"),
    [
        (add, 2.0, 3.0, 5.0, Operation.ADD),
        (subtract, 7.0, 3.0, 4.0, Operation.SUBTRACT),
        (multiply, 2.5, 4.0, 10.0, Operation.MULTIPLY),
        (divide, 9.0, 2.0, 4.5, Operation.DIVIDE),
    ],
)
def test_operations_return_calculation(
    operation: OperationFunction,
    left: float,
    right: float,
    expected_result: float,
    expected_operation: Operation,
) -> None:
    """Every public operation returns the documented calculation interface."""
    calculation = operation(left, right)

    assert isinstance(calculation, Calculation)
    assert calculation.left == left
    assert calculation.right == right
    assert calculation.result == expected_result
    assert calculation.operation is expected_operation


@pytest.mark.public_api(MODULE_NAME, "add")
def test_add_normalizes_operands_before_calculating() -> None:
    """The stored operands and result use the same float representation."""
    calculation = add(9_007_199_254_740_993, -9_007_199_254_740_992)

    assert calculation.left + calculation.right == calculation.result


@pytest.mark.public_api(MODULE_NAME, "divide")
def test_divide_rejects_zero_divisor() -> None:
    """Division has a clear public failure mode for zero divisors."""
    with pytest.raises(ValueError, match="Cannot divide by zero"):
        divide(1.0, 0.0)
