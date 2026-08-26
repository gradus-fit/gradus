# SPDX-License-Identifier: AGPL-3.0-or-later

"""Public interface for server configuration."""

from .src._configuration import get_boolean, get_float, get_integer, get_string
from .src.models import (
    ConfigurationError,
    ConfigurationValueError,
    ConfigurationVariable,
    UnknownConfigurationVariableError,
)

__all__ = [
    "ConfigurationError",
    "ConfigurationValueError",
    "ConfigurationVariable",
    "UnknownConfigurationVariableError",
    "get_boolean",
    "get_float",
    "get_integer",
    "get_string",
]
