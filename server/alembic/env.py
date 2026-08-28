# SPDX-License-Identifier: AGPL-3.0-or-later

"""Alembic environment using the connection supplied by the migrations module."""

from logging.config import fileConfig
from typing import cast

from alembic import context
from sqlalchemy.engine import Connection

config = context.config

if config.config_file_name is not None:
    fileConfig(config.config_file_name)


def run_migrations_offline() -> None:
    """Reject URL-based migration execution."""
    raise RuntimeError("Alembic migrations require a supplied database connection")


def run_migrations_online() -> None:
    """Apply migrations through the connection supplied by the caller."""
    connection = cast(
        Connection, config.attributes["connection"]  # type: ignore[misc] # Alembic exposes untyped configuration attributes.
    )
    context.configure(connection=connection)

    with context.begin_transaction():
        context.run_migrations()


if context.is_offline_mode():
    run_migrations_offline()
else:
    run_migrations_online()
