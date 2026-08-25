# SPDX-License-Identifier: AGPL-3.0-or-later

"""Pytest policy checks shared by all server modules."""

import ast
from pathlib import Path
from typing import cast

import pytest

_REQUIRED_EXPORTS_KEY: pytest.StashKey[dict[str, set[str]]] = pytest.StashKey()
_EXERCISED_EXPORTS: dict[str, set[str]] = {}
_MARKED_EXPORTS_BY_NODEID: dict[str, list[tuple[str, ...]]] = {}


def pytest_configure(config: pytest.Config) -> None:
    """Register the marker used to declare exercised public exports."""
    config.addinivalue_line(
        "markers",
        "public_api(module, *exports): declares public module exports exercised by a test",
    )
    config.stash[_REQUIRED_EXPORTS_KEY] = _discover_public_exports(
        config.rootpath / "src"
    )


def pytest_collection_modifyitems(
    config: pytest.Config, items: list[pytest.Item]
) -> None:
    """Associate each collected test with its declared public exports."""
    _EXERCISED_EXPORTS.clear()
    _MARKED_EXPORTS_BY_NODEID.clear()
    for item in items:
        markers = [
            _marker_arguments(marker)
            for marker in item.iter_markers(name="public_api")
        ]
        if markers:
            _MARKED_EXPORTS_BY_NODEID[item.nodeid] = markers


def pytest_runtest_logreport(report: pytest.TestReport) -> None:
    """Credit public exports only when a marked test passes."""
    if report.when != "call" or not report.passed:
        return

    for module, *exports in _MARKED_EXPORTS_BY_NODEID.get(report.nodeid, []):
        _EXERCISED_EXPORTS.setdefault(module, set()).update(exports)


def pytest_sessionfinish(session: pytest.Session, exitstatus: int | pytest.ExitCode) -> None:
    """Fail when a module export has no successful public-interface test."""
    required_exports = session.config.stash.get(_REQUIRED_EXPORTS_KEY, None)
    if required_exports is None:
        return

    missing_exports = {
        module: exports - _EXERCISED_EXPORTS.get(module, set())
        for module, exports in required_exports.items()
        if exports - _EXERCISED_EXPORTS.get(module, set())
    }
    if missing_exports:
        details = "; ".join(
            f"{module}: {', '.join(sorted(exports))}"
            for module, exports in sorted(missing_exports.items())
        )
        raise pytest.UsageError(f"Missing successful public API tests for {details}")


def _discover_public_exports(source_root: Path) -> dict[str, set[str]]:
    """Return declared public exports for every server module."""
    modules: dict[str, set[str]] = {}
    for module_path in source_root.iterdir():
        if not module_path.is_dir() or module_path.name == "__pycache__":
            continue

        init_file = module_path / "__init__.py"
        if not init_file.is_file():
            raise pytest.UsageError(f"Server module {module_path} must contain __init__.py")
        modules[f"src.{module_path.name}"] = _read_public_exports(init_file)
    return modules


def _read_public_exports(init_file: Path) -> set[str]:
    """Read a module's sole literal ``__all__`` declaration without importing it."""
    module = ast.parse(init_file.read_text(encoding="utf-8"), filename=str(init_file))
    assignments: list[ast.expr] = []

    for statement in module.body:
        if isinstance(statement, ast.Assign) and any(
            _is_public_exports_name(target) for target in statement.targets
        ):
            assignments.append(statement.value)
        elif isinstance(statement, ast.AnnAssign) and _is_public_exports_name(
            statement.target
        ):
            if statement.value is None:
                raise pytest.UsageError(f"{init_file} must assign a value to __all__")
            assignments.append(statement.value)
        elif isinstance(statement, ast.AugAssign) and _is_public_exports_name(
            statement.target
        ):
            raise pytest.UsageError(f"{init_file} must not modify __all__")
        elif _modifies_public_exports(statement):
            raise pytest.UsageError(f"{init_file} must not modify __all__")

    if len(assignments) != 1:
        raise pytest.UsageError(
            f"{init_file} must declare exactly one literal __all__ list or tuple"
        )
    return _string_sequence(assignments[0], init_file)


def _is_public_exports_name(node: ast.expr) -> bool:
    """Return whether an AST node references ``__all__`` directly."""
    return isinstance(node, ast.Name) and node.id == "__all__"


def _modifies_public_exports(statement: ast.stmt) -> bool:
    """Return whether a statement calls a method on ``__all__``."""
    return (
        isinstance(statement, ast.Expr)
        and isinstance(statement.value, ast.Call)
        and isinstance(statement.value.func, ast.Attribute)
        and _is_public_exports_name(statement.value.func.value)
    )


def _string_sequence(value: ast.expr, init_file: Path) -> set[str]:
    """Validate and return literal string exports."""
    if not isinstance(value, ast.List | ast.Tuple):
        raise pytest.UsageError(f"{init_file} must declare __all__ as a list or tuple")

    exports: set[str] = set()
    for element in value.elts:
        if not isinstance(element, ast.Constant) or not isinstance(element.value, str):
            raise pytest.UsageError(f"{init_file} must declare __all__ with string values")
        exports.add(element.value)
    return exports


def _marker_arguments(marker: pytest.Mark) -> tuple[str, ...]:
    """Validate and return one ``public_api`` marker's arguments."""
    arguments = cast(tuple[str, ...], marker.args)
    if not arguments or not all(isinstance(argument, str) for argument in arguments):
        raise pytest.UsageError("public_api marker arguments must be module and export names")
    return arguments
