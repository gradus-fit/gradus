# SPDX-License-Identifier: AGPL-3.0-or-later

from pathlib import Path
import shutil

import pytest

pytest_plugins = ("pytester",)

_SERVER_ROOT = Path(__file__).parents[1]
_POLICY_PLUGIN = _SERVER_ROOT / "conftest.py"
_TEMPLATE_DIRECTORY = _SERVER_ROOT / "template"


def test_copied_template_satisfies_public_api_policy(
    pytester: pytest.Pytester,
) -> None:
    """A copied module keeps its portable test imports and public-API marker."""
    source_root = pytester.path / "src"
    source_root.mkdir()
    (source_root / "__init__.py").write_text("", encoding="utf-8")
    shutil.copytree(_TEMPLATE_DIRECTORY, source_root / "example")
    pytester.makeconftest(_POLICY_PLUGIN.read_text(encoding="utf-8"))

    result = pytester.runpytest_subprocess()

    result.assert_outcomes(passed=1)


def test_policy_rejects_missing_public_api_marker(pytester: pytest.Pytester) -> None:
    """An export without a passing marked test fails the test session."""
    _write_project(pytester, '__all__ = ["run"]\n')
    pytester.makepyfile("def test_run() -> None:\n    pass\n")

    result = pytester.runpytest()

    assert result.ret is pytest.ExitCode.USAGE_ERROR
    result.stderr.fnmatch_lines(["*Missing successful public API tests for src.example: run*"])


def test_policy_rejects_skipped_public_api_test(pytester: pytest.Pytester) -> None:
    """A skipped test cannot satisfy an export's public-interface requirement."""
    _write_project(pytester, '__all__ = ["run"]\n')
    pytester.makepyfile(
        """\
import pytest


@pytest.mark.public_api("src.example", "run")
@pytest.mark.skip
def test_run() -> None:
    pass
"""
    )

    result = pytester.runpytest()

    assert result.ret is pytest.ExitCode.USAGE_ERROR
    result.stderr.fnmatch_lines(["*Missing successful public API tests for src.example: run*"])


def test_policy_rejects_modified_public_exports(pytester: pytest.Pytester) -> None:
    """A module cannot modify its literal public-interface declaration."""
    _write_project(pytester, '__all__ = ["run"]\n__all__.append("other")\n')

    result = pytester.runpytest()

    assert result.ret is pytest.ExitCode.USAGE_ERROR
    result.stderr.fnmatch_lines(["*must not modify __all__*"])


def test_policy_rejects_module_without_init_file(pytester: pytest.Pytester) -> None:
    """Every first-level server module must declare a public interface file."""
    _write_project(pytester, '__all__ = ["run"]\n')
    (pytester.path / "src" / "incomplete").mkdir()

    result = pytester.runpytest()

    assert result.ret is pytest.ExitCode.USAGE_ERROR
    result.stderr.fnmatch_lines(["*Server module *incomplete must contain __init__.py*"])


def _write_project(pytester: pytest.Pytester, init_contents: str) -> None:
    """Create a temporary server module governed by the public-API policy."""
    module_path = pytester.path / "src" / "example"
    module_path.mkdir(parents=True)
    (module_path / "__init__.py").write_text(init_contents, encoding="utf-8")
    pytester.makeconftest(_POLICY_PLUGIN.read_text(encoding="utf-8"))
