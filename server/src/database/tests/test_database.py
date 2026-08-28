# SPDX-License-Identifier: AGPL-3.0-or-later

"""Tests for the database module's public interface."""

from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import pytest

from src.database import (
    DatabaseConfigurationError,
    Field,
    SQLModel,
    UnsupportedDatabaseBackendError,
    get_engine,
    select,
    session,
    text,
)
from src.config import ConfigurationVariable

MODULE_NAME = "src.database"


class DatabaseRecord(SQLModel, table=True):
    """Test-only table declared by a database consumer."""

    id: int | None = Field(default=None, primary_key=True)
    email: str


@pytest.fixture  # type: ignore[misc] # pytest's overloaded fixture decorator is dynamic.
def database_url(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> str:
    """Configure an isolated SQLite database and create consumer-owned DDL."""
    url = f"sqlite:///{tmp_path / 'database.sqlite3'}"
    monkeypatch.setenv(ConfigurationVariable.DATABASE_URL, url)
    SQLModel.metadata.create_all(get_engine())  # type: ignore[misc] # SQLModel metadata is dynamic.
    return url


@pytest.mark.public_api(MODULE_NAME, "Field", "SQLModel", "get_engine")
def test_get_engine_is_reused_without_creating_a_database_file(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Engine construction is URL-cached and does not connect or create a file."""
    database_file = tmp_path / "not-created.sqlite3"
    monkeypatch.setenv(ConfigurationVariable.DATABASE_URL, f"sqlite:///{database_file}")

    first_engine = get_engine()
    second_engine = get_engine()

    assert first_engine.dialect.name == "sqlite"
    assert first_engine is second_engine
    assert not database_file.exists()


@pytest.mark.public_api(MODULE_NAME, "get_engine")
def test_get_engine_is_reused_during_concurrent_initialization(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Concurrent first access creates and returns one engine for a URL."""
    database_file = tmp_path / "concurrent.sqlite3"
    monkeypatch.setenv(ConfigurationVariable.DATABASE_URL, f"sqlite:///{database_file}")

    with ThreadPoolExecutor(max_workers=4) as executor:
        futures = [executor.submit(get_engine) for _ in range(4)]
        engines = [future.result() for future in futures]

    assert all(engine is engines[0] for engine in engines)


@pytest.mark.public_api(MODULE_NAME, "Field", "SQLModel", "select", "session")
def test_in_memory_database_is_available_across_threads(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """The documented in-memory URL shares a schema across server threads."""
    monkeypatch.setenv(ConfigurationVariable.DATABASE_URL, "sqlite://")
    SQLModel.metadata.create_all(get_engine())  # type: ignore[misc] # SQLModel metadata is dynamic.

    def insert_record() -> int:
        with session() as database_session:
            record = DatabaseRecord(email="thread@example.com")
            database_session.add(record)
            database_session.commit()
            database_session.refresh(record)
            assert record.id is not None
            return record.id

    with ThreadPoolExecutor(max_workers=1) as executor:
        record_id = executor.submit(insert_record).result()

    with session() as database_session:
        found = database_session.exec(select(DatabaseRecord)).one()  # type: ignore[misc] # SQLModel's select typing is dynamic.

    assert found.id == record_id


@pytest.mark.public_api(MODULE_NAME, "Field", "SQLModel", "select", "session")
def test_session_supports_committed_crud(database_url: str) -> None:
    """Consumers explicitly commit inserts, updates, and deletes."""
    with session() as database_session:
        record = DatabaseRecord(email="person@example.com")
        database_session.add(record)
        database_session.commit()
        database_session.refresh(record)

        found = database_session.exec(
            select(DatabaseRecord).where(  # type: ignore[misc] # SQLModel's select typing is dynamic.
                DatabaseRecord.email == record.email  # type: ignore[misc] # SQLModel table attributes are dynamic.
            )
        ).one()
        found.email = "updated@example.com"
        database_session.add(found)
        database_session.commit()
        database_session.delete(found)
        database_session.commit()

        remaining = database_session.exec(select(DatabaseRecord)).all()  # type: ignore[misc] # SQLModel's select typing is dynamic.

    assert record.id is not None
    assert remaining == []


@pytest.mark.public_api(MODULE_NAME, "session", "text")
def test_session_executes_parameterized_sql(database_url: str) -> None:
    """Raw SQL uses bound parameters through the public text helper."""
    email = "person@example.com"
    with session() as database_session:
        database_session.add(DatabaseRecord(email=email))
        database_session.commit()

        result = database_session.exec(  # type: ignore[call-overload, misc] # SQLModel's stubs omit TextClause support.
            text("SELECT email FROM databaserecord WHERE email = :email"),
            params={"email": email},
        ).one()

    assert result[0] == email  # type: ignore[misc] # SQLModel's raw SQL result is dynamic.


@pytest.mark.public_api(MODULE_NAME, "session")
def test_session_closes_when_its_context_exits(database_url: str) -> None:
    """Context exit closes the session and clears its active transaction state."""
    with session() as database_session:
        record = DatabaseRecord(email="uncommitted@example.com")
        database_session.add(record)
        database_session.flush()

        assert database_session.identity_map
        assert database_session.in_transaction()

    assert not database_session.identity_map
    assert not database_session.in_transaction()


@pytest.mark.public_api(
    MODULE_NAME,
    "DatabaseConfigurationError",
    "UnsupportedDatabaseBackendError",
    "get_engine",
)
def test_get_engine_rejects_postgresql_until_that_backend_is_enabled(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """PostgreSQL rejection remains explicitly temporary in Phase 1."""
    monkeypatch.setenv(
        ConfigurationVariable.DATABASE_URL,
        "postgresql+psycopg://user:password@localhost/database",
    )

    with pytest.raises(UnsupportedDatabaseBackendError, match="PostgreSQL support is planned") as error:
        get_engine()

    assert isinstance(error.value, DatabaseConfigurationError)


@pytest.mark.public_api(MODULE_NAME, "DatabaseConfigurationError", "get_engine")
@pytest.mark.parametrize(
    "database_url",
    ["sqlite://localhost/database.sqlite3", "sqlite://host:invalid/database.sqlite3"],
)
def test_get_engine_rejects_unusable_sqlite_urls(
    monkeypatch: pytest.MonkeyPatch, database_url: str
) -> None:
    """Malformed SQLite URLs use the module's common configuration error."""
    monkeypatch.setenv(ConfigurationVariable.DATABASE_URL, database_url)

    with pytest.raises(DatabaseConfigurationError):
        get_engine()


@pytest.mark.public_api(MODULE_NAME, "get_engine")
def test_get_engine_preserves_missing_configuration_error(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Missing URLs retain the configuration module's public error type."""
    monkeypatch.delenv(ConfigurationVariable.DATABASE_URL, raising=False)
    monkeypatch.chdir(tmp_path)

    with pytest.raises(ValueError, match="DATABASE_URL is not set") as error:
        get_engine()

    assert type(error.value).__name__ == "ConfigurationValueError"
