# SPDX-License-Identifier: AGPL-3.0-or-later

"""Private implementation of configuration readers."""

import os

from dotenv import dotenv_values, find_dotenv

from .models import (
    ConfigurationValueError,
    ConfigurationVariable,
    UnknownConfigurationVariableError,
)

_TRUE_VALUES = frozenset({"1", "true", "yes", "on"})
_FALSE_VALUES = frozenset({"0", "false", "no", "off"})


def get_string(variable: ConfigurationVariable) -> str:
    """Return a configured string value."""
    _validate_variable(variable)
    process_value = os.getenv(variable)
    if process_value is not None:
        return process_value

    try:
        dotenv_value = dotenv_values(
            find_dotenv(usecwd=True), interpolate=False
        ).get(variable)
    except (OSError, UnicodeError) as error:
        raise ConfigurationValueError("Cannot read .env configuration") from error
    if dotenv_value is not None:
        return dotenv_value

    raise ConfigurationValueError(f"Configuration variable {variable} is not set")


def get_integer(variable: ConfigurationVariable) -> int:
    """Return a configured integer value."""
    value = get_string(variable)
    try:
        return int(value)
    except ValueError as error:
        raise ConfigurationValueError(
            f"Configuration variable {variable} must be an integer"
        ) from error


def get_float(variable: ConfigurationVariable) -> float:
    """Return a configured floating-point value."""
    value = get_string(variable)
    try:
        return float(value)
    except ValueError as error:
        raise ConfigurationValueError(
            f"Configuration variable {variable} must be a floating-point number"
        ) from error


def get_boolean(variable: ConfigurationVariable) -> bool:
    """Return a configured boolean value."""
    value = get_string(variable).casefold()
    if value in _TRUE_VALUES:
        return True
    if value in _FALSE_VALUES:
        return False
    raise ConfigurationValueError(
        f"Configuration variable {variable} must be a boolean"
    )


def _validate_variable(variable: ConfigurationVariable) -> None:
    """Reject values that do not belong to the configuration allowlist."""
    if not isinstance(variable, ConfigurationVariable):
        raise UnknownConfigurationVariableError(
            f"Configuration variable {variable!r} is not allowed"
        )
