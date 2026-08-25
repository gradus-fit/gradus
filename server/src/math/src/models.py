# SPDX-License-Identifier: AGPL-3.0-or-later

"""Typed data structures returned by the math module."""

from dataclasses import dataclass
from enum import StrEnum


class Operation(StrEnum):
    """A supported arithmetic operation."""

    ADD = "add"
    SUBTRACT = "subtract"
    MULTIPLY = "multiply"
    DIVIDE = "divide"


@dataclass(frozen=True, slots=True, kw_only=True)
class Calculation:
    """The operands and result of one arithmetic operation."""

    operation: Operation
    left: float
    right: float
    result: float
