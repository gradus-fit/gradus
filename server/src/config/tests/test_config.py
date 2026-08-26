# SPDX-License-Identifier: AGPL-3.0-or-later

"""Tests for the configuration module's public interface."""

from pathlib import Path

import pytest

from .. import (
    ConfigurationError,
    ConfigurationValueError,
    ConfigurationVariable,
    UnknownConfigurationVariableError,
    get_boolean,
    get_float,
    get_integer,
    get_string,
)

MODULE_NAME = __name__.rsplit(".tests.", maxsplit=1)[0]


@pytest.mark.public_api(MODULE_NAME, "ConfigurationVariable", "get_string")
def test_get_string_reads_an_allowlisted_process_variable(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Process environment values are returned for allowed variables."""
    monkeypatch.setenv(ConfigurationVariable.SERVER_HOST, "0.0.0.0")

    assert get_string(ConfigurationVariable.SERVER_HOST) == "0.0.0.0"


@pytest.mark.public_api(MODULE_NAME, "get_string")
def test_get_string_reads_a_dotenv_value(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """A discovered .env file provides values not set in the process."""
    dotenv_file = tmp_path / ".env"
    dotenv_file.write_text("SERVER_HOST=localhost\n", encoding="utf-8")
    monkeypatch.delenv(ConfigurationVariable.SERVER_HOST, raising=False)
    monkeypatch.chdir(tmp_path)

    assert get_string(ConfigurationVariable.SERVER_HOST) == "localhost"


@pytest.mark.public_api(MODULE_NAME, "get_string")
def test_get_string_prefers_a_process_value_over_dotenv(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Process configuration overrides a value from .env."""
    dotenv_file = tmp_path / ".env"
    dotenv_file.write_text("SERVER_HOST=from-dotenv\n", encoding="utf-8")
    monkeypatch.setenv(ConfigurationVariable.SERVER_HOST, "from-process")
    monkeypatch.chdir(tmp_path)

    assert get_string(ConfigurationVariable.SERVER_HOST) == "from-process"


@pytest.mark.public_api(MODULE_NAME, "get_string")
def test_get_string_does_not_interpolate_disallowed_variables(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Dotenv interpolation cannot expose variables outside the allowlist."""
    dotenv_file = tmp_path / ".env"
    dotenv_file.write_text("SERVER_HOST=${HOME}\n", encoding="utf-8")
    monkeypatch.delenv(ConfigurationVariable.SERVER_HOST, raising=False)
    monkeypatch.setenv("HOME", "not-allowed")
    monkeypatch.chdir(tmp_path)

    assert get_string(ConfigurationVariable.SERVER_HOST) == "${HOME}"


@pytest.mark.public_api(MODULE_NAME, "get_string", "UnknownConfigurationVariableError")
def test_get_string_rejects_a_variable_outside_the_allowlist() -> None:
    """Raw environment names cannot bypass the enum allowlist."""
    with pytest.raises(UnknownConfigurationVariableError, match="not allowed"):
        get_string("HOME")  # type: ignore[arg-type]


@pytest.mark.public_api(MODULE_NAME, "ConfigurationError", "ConfigurationValueError")
def test_get_string_rejects_missing_values(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Absent allowlisted values have a clear public error."""
    monkeypatch.delenv(ConfigurationVariable.SERVER_PORT, raising=False)
    monkeypatch.chdir(tmp_path)

    with pytest.raises(ConfigurationValueError, match="SERVER_PORT is not set") as error:
        get_string(ConfigurationVariable.SERVER_PORT)

    assert isinstance(error.value, ConfigurationError)


@pytest.mark.public_api(MODULE_NAME, "get_string", "ConfigurationValueError")
def test_get_string_wraps_dotenv_read_errors(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Unreadable dotenv content raises a public configuration error."""
    dotenv_file = tmp_path / ".env"
    dotenv_file.write_bytes(b"SERVER_HOST=\xff")
    monkeypatch.delenv(ConfigurationVariable.SERVER_HOST, raising=False)
    monkeypatch.chdir(tmp_path)

    with pytest.raises(ConfigurationValueError, match="Cannot read .env configuration"):
        get_string(ConfigurationVariable.SERVER_HOST)


@pytest.mark.public_api(MODULE_NAME, "get_integer", "get_float")
def test_numeric_getters_parse_values(monkeypatch: pytest.MonkeyPatch) -> None:
    """Numeric getters return their documented Python numeric types."""
    monkeypatch.setenv(ConfigurationVariable.SERVER_PORT, "8000")

    assert get_integer(ConfigurationVariable.SERVER_PORT) == 8000
    assert get_float(ConfigurationVariable.SERVER_PORT) == 8000.0


@pytest.mark.public_api(MODULE_NAME, "get_integer", "get_float")
def test_numeric_getters_reject_invalid_values(monkeypatch: pytest.MonkeyPatch) -> None:
    """Numeric getters reject configuration values of the wrong type."""
    monkeypatch.setenv(ConfigurationVariable.SERVER_PORT, "not-a-number")

    with pytest.raises(ConfigurationValueError, match="must be an integer"):
        get_integer(ConfigurationVariable.SERVER_PORT)
    with pytest.raises(ConfigurationValueError, match="must be a floating-point number"):
        get_float(ConfigurationVariable.SERVER_PORT)


@pytest.mark.public_api(MODULE_NAME, "get_boolean")
@pytest.mark.parametrize(
    ("value", "expected"),
    [("TRUE", True), ("yes", True), ("0", False), ("off", False)],
)
def test_get_boolean_parses_documented_values(
    monkeypatch: pytest.MonkeyPatch, value: str, expected: bool
) -> None:
    """Boolean getter accepts every documented spelling."""
    monkeypatch.setenv(ConfigurationVariable.SERVER_PORT, value)

    assert get_boolean(ConfigurationVariable.SERVER_PORT) is expected


@pytest.mark.public_api(MODULE_NAME, "get_boolean")
def test_get_boolean_rejects_invalid_values(monkeypatch: pytest.MonkeyPatch) -> None:
    """Boolean getter rejects values outside its documented spellings."""
    monkeypatch.setenv(ConfigurationVariable.SERVER_PORT, "sometimes")

    with pytest.raises(ConfigurationValueError, match="must be a boolean"):
        get_boolean(ConfigurationVariable.SERVER_PORT)
