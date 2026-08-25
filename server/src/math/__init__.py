# SPDX-License-Identifier: AGPL-3.0-or-later

"""Public interface for arithmetic calculations."""

from .src._operations import add, divide, multiply, subtract
from .src.models import Calculation, Operation

__all__ = [
    "Calculation",
    "Operation",
    "add",
    "divide",
    "multiply",
    "subtract",
]
