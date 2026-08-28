# SPDX-License-Identifier: AGPL-3.0-or-later

"""Alembic migration execution and revision creation."""

import argparse
from pathlib import Path

from alembic import command
from alembic.config import Config
from sqlalchemy.engine import Connection

from src.database import get_engine

_PROJECT_ROOT = Path(__file__).resolve().parents[3]
_ALEMBIC_INI_PATH = _PROJECT_ROOT / "alembic.ini"
_ALEMBIC_PATH = _PROJECT_ROOT / "alembic"


def upgrade() -> None:
    """Apply every reviewed Alembic revision through the configured engine."""
    with get_engine().connect() as connection:
        config = _build_config()
        config.attributes["connection"] = connection  # type: ignore[misc] # Alembic exposes untyped configuration attributes.
        command.upgrade(config, "head")


def main() -> None:
    """Run the private developer CLI."""
    parser = argparse.ArgumentParser(description="Manage Server Alembic revisions.")
    subcommands = parser.add_subparsers(dest="command", required=True)
    revision_parser = subcommands.add_parser(
        "revision", help="Create an empty reviewed migration revision."
    )
    revision_parser.add_argument("--message", required=True)
    subcommands.add_parser("upgrade", help="Apply all reviewed migration revisions.")
    arguments = parser.parse_args()

    if arguments.command == "revision":  # type: ignore[misc] # argparse Namespace attributes are untyped.
        command.revision(
            _build_config(),
            message=arguments.message,  # type: ignore[misc] # argparse Namespace attributes are untyped.
        )
    else:
        upgrade()


def _build_config() -> Config:
    """Build Alembic configuration for this application's revision history."""
    config = Config(str(_ALEMBIC_INI_PATH))
    config.set_main_option("script_location", str(_ALEMBIC_PATH))
    return config
