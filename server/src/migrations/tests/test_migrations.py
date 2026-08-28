# SPDX-License-Identifier: AGPL-3.0-or-later

"""Tests for the migrations module's public interface and private CLI."""

import os
import shutil
import sqlite3
import subprocess
import sys
from pathlib import Path
from typing import cast

import pytest

from src.migrations import upgrade

MODULE_NAME = "src.migrations"
_BASELINE_REVISION = "0001_baseline"


@pytest.mark.public_api(MODULE_NAME, "upgrade")
def test_upgrade_applies_the_baseline_and_is_idempotent(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """An empty database reaches head and a later invocation is a no-op."""
    database_file = tmp_path / "migrations.sqlite3"
    monkeypatch.setenv("DATABASE_URL", f"sqlite:///{database_file}")

    upgrade()
    first_version = _database_version(database_file)
    upgrade()

    assert first_version == _BASELINE_REVISION
    assert _database_version(database_file) == _BASELINE_REVISION


@pytest.mark.public_api(MODULE_NAME, "upgrade")
def test_upgrade_preserves_missing_database_url_error(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Configuration errors abort migration before Alembic is invoked."""
    monkeypatch.delenv("DATABASE_URL", raising=False)
    monkeypatch.chdir(tmp_path)

    with pytest.raises(ValueError, match="DATABASE_URL is not set") as error:
        upgrade()

    assert type(error.value).__name__ == "ConfigurationValueError"


@pytest.mark.public_api(MODULE_NAME, "upgrade")
def test_upgrade_propagates_alembic_failures(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Alembic failures prevent successful upgrade completion."""
    monkeypatch.setenv("DATABASE_URL", f"sqlite:///{tmp_path / 'migrations.sqlite3'}")

    def fail_upgrade(*_arguments: str) -> None:
        raise RuntimeError("migration failed")

    monkeypatch.setattr("alembic.command.upgrade", fail_upgrade)

    with pytest.raises(RuntimeError, match="migration failed"):
        upgrade()


def test_revision_cli_creates_an_empty_revision_in_an_isolated_history(
    tmp_path: Path,
) -> None:
    """The CLI writes a new empty revision without touching the real history."""
    isolated_server = tmp_path / "server"
    source_server = Path(__file__).resolve().parents[3]
    source_versions = sorted((source_server / "alembic" / "versions").glob("*.py"))
    shutil.copytree(source_server / "src", isolated_server / "src")
    shutil.copytree(source_server / "alembic", isolated_server / "alembic")
    shutil.copy2(source_server / "alembic.ini", isolated_server / "alembic.ini")

    subprocess.run(
        [
            sys.executable,
            "-m",
            "src.migrations",
            "revision",
            "--message",
            "create users",
        ],
        cwd=isolated_server,
        check=True,
        capture_output=True,
        text=True,
    )

    revisions = sorted((isolated_server / "alembic" / "versions").glob("*.py"))
    assert sorted((source_server / "alembic" / "versions").glob("*.py")) == source_versions
    assert len(revisions) == 2
    generated_revision = next(
        revision for revision in revisions if revision.name != "0001_baseline_baseline.py"
    )
    content = generated_revision.read_text(encoding="utf-8")
    assert "def upgrade() -> None:" in content
    assert "def downgrade() -> None:" in content
    assert "    pass" in content


def test_upgrade_cli_applies_the_baseline(tmp_path: Path) -> None:
    """The operational CLI applies the configured database's migration history."""
    isolated_server = tmp_path / "server"
    source_server = Path(__file__).resolve().parents[3]
    database_file = tmp_path / "migrations.sqlite3"
    shutil.copytree(source_server / "src", isolated_server / "src")
    shutil.copytree(source_server / "alembic", isolated_server / "alembic")
    shutil.copy2(source_server / "alembic.ini", isolated_server / "alembic.ini")
    environment = os.environ.copy()
    environment["DATABASE_URL"] = f"sqlite:///{database_file}"

    subprocess.run(
        [sys.executable, "-m", "src.migrations", "upgrade"],
        cwd=isolated_server,
        check=True,
        capture_output=True,
        text=True,
        env=environment,
    )

    assert _database_version(database_file) == _BASELINE_REVISION


def _database_version(database_file: Path) -> str:
    """Return the Alembic version installed in the isolated SQLite database."""
    with sqlite3.connect(database_file) as connection:
        row = cast(
            tuple[str] | None,
            connection.execute(
                "SELECT version_num FROM alembic_version"
            ).fetchone(),
        )

    assert row is not None
    return row[0]
