# SPDX-License-Identifier: AGPL-3.0-or-later

"""Public configuration types."""

from enum import StrEnum


class ConfigurationVariable(StrEnum):
    """Environment variables the server is permitted to read."""

    DATABASE_URL = "DATABASE_URL"
    SERVER_HOST = "SERVER_HOST"
    SERVER_PORT = "SERVER_PORT"


class ConfigurationError(ValueError):
    """Base error raised when configuration cannot be read."""


class UnknownConfigurationVariableError(ConfigurationError):
    """Raised when a caller requests a variable outside the allowlist."""


class ConfigurationValueError(ConfigurationError):
    """Raised when a configuration value is missing or has an invalid type."""
