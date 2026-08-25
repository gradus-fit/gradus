# SPDX-License-Identifier: AGPL-3.0-or-later

from pathlib import Path
import subprocess
import sys


def test_strict_mypy_policy_rejects_unsafe_annotations(tmp_path: Path) -> None:
    """The type checker rejects the unsafe annotations covered by this policy."""
    fixture = tmp_path / "unsafe_annotations.py"
    fixture.write_text(
        """\
from builtins import object as GenericObject
from typing import Any

unsafe_any: Any = 1
imprecise_value: GenericObject = 1
untyped_mapping: dict = {}


def missing_annotations(value):
    return value
""",
        encoding="utf-8",
    )

    result = subprocess.run(
        [sys.executable, "-m", "mypy", str(fixture)],
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode != 0
    output = result.stdout + result.stderr
    assert 'Explicit "Any" is not allowed' in output
    assert "Use a precise type instead of the generic 'object' type." in output
    assert 'Missing type arguments for generic type "dict"' in output
    assert "Function is missing a type annotation" in output
