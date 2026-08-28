# SPDX-License-Identifier: AGPL-3.0-or-later

"""Private engine and session construction."""

from collections.abc import Iterator
from contextlib import contextmanager
from threading import RLock

from sqlalchemy import create_engine
from sqlalchemy.engine import Engine, URL, make_url
from sqlalchemy.exc import ArgumentError
from sqlalchemy.pool import StaticPool
from sqlmodel import Session

from src.config import ConfigurationVariable, get_string

from .models import DatabaseConfigurationError, UnsupportedDatabaseBackendError

_ENGINES_BY_URL: dict[str, Engine] = {}
_ENGINES_LOCK = RLock()


def get_engine() -> Engine:
    """Return the reusable engine for the configured SQLite database URL."""
    database_url = get_string(ConfigurationVariable.DATABASE_URL)
    with _ENGINES_LOCK:
        engine = _ENGINES_BY_URL.get(database_url)
        if engine is not None:
            return engine

        url = _validate_sqlite_url(database_url)
        engine = _create_sqlite_engine(database_url, url)
        _ENGINES_BY_URL[database_url] = engine
        return engine


@contextmanager
def session() -> Iterator[Session]:
    """Open a session that callers commit or roll back explicitly."""
    database_session = Session(get_engine())
    try:
        yield database_session
    finally:
        database_session.close()


def _validate_sqlite_url(database_url: str) -> URL:
    """Validate the Phase 1 SQLite-only URL contract without connecting."""
    try:
        url = make_url(database_url)
        has_connection_target = any(
            value is not None
            for value in (url.username, url.password, url.host, url.port)
        )
    except (ArgumentError, ValueError) as error:  # type: ignore[misc] # SQLAlchemy lacks strict stubs.
        raise DatabaseConfigurationError(
            "DATABASE_URL must be a valid SQLAlchemy database URL"
        ) from error

    if url.drivername != "sqlite":
        raise UnsupportedDatabaseBackendError(
            "Database backend is not enabled in Phase 1; PostgreSQL support is planned"
        )
    if has_connection_target:
        raise DatabaseConfigurationError(
            "DATABASE_URL must use a SQLite URL without credentials or a host"
        )
    return url


def _create_sqlite_engine(database_url: str, url: URL) -> Engine:
    """Construct a SQLite engine with private, dialect-specific options."""
    try:
        if url.database in (None, ":memory:"):
            return create_engine(
                database_url,
                connect_args={"check_same_thread": False},  # type: ignore[misc] # SQLAlchemy's options use Any.
                echo=False,
                poolclass=StaticPool,
            )
        return create_engine(
            database_url,
            connect_args={"check_same_thread": False},  # type: ignore[misc] # SQLAlchemy's options use Any.
            echo=False,
        )
    except (ArgumentError, TypeError, ValueError) as error:  # type: ignore[misc] # SQLAlchemy lacks strict stubs.
        raise DatabaseConfigurationError(
            "DATABASE_URL must be a usable SQLite SQLAlchemy URL"
        ) from error
