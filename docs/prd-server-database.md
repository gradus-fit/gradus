# PRD: SQLModel database Server module

## Status

Proposed. Implement only after `docs/prd-server-config-database-url.md` is complete. This creates one Server module, `server/src/database`, and depends on the existing `config` module.

## Problem

Future Server modules, beginning with registration/authentication, need a consistent way to define SQLModel tables, initialize a schema, and perform transactional CRUD or parameterized SQL. They must not each create engines or read connection settings directly.

## Goal

Provide a small synchronous SQLModel-backed `database` module with a backend-neutral public interface. Phase 1 supports SQLite; PostgreSQL support follows in the near term without changing consumer-facing table, schema-creation, session, or query APIs. Consumer modules own their table models and queries.

## Scope

### Phase 1

- SQLite only, using the Python standard-library SQLite driver through SQLModel/SQLAlchemy.
- Connection URL from `get_string(ConfigurationVariable.DATABASE_URL)`.
- SQLModel table declaration, schema creation for registered tables, sessions, CRUD, and parameterized SQL.
- No application-specific tables or registration/authentication behavior.

### PostgreSQL readiness

- Keep the public API and consumer table models backend-neutral; do not expose SQLite connection flags, SQLite-only SQL, or SQLite-specific column types as part of the module contract.
- Treat `DATABASE_URL` as a SQLAlchemy URL, not as a SQLite filename. The Phase 1 accepted forms are `sqlite://` and `sqlite:///...`; PostgreSQL forms are intentionally a subsequent capability.
- Isolate dialect-specific engine options privately. `check_same_thread=False` applies only to SQLite and must never be passed to a PostgreSQL engine.
- Do not make consumers branch on a database backend. The follow-on PostgreSQL work changes only the database module's private engine configuration, dependencies, tests, README, and supported-URL validation.

## Prerequisite and dependencies

1. Complete `docs/prd-server-config-database-url.md` first.
2. Add the current maintained `sqlmodel` release compatible with the repository's Python 3.14 requirement to `server/pyproject.toml`, then regenerate `server/uv.lock` with `uv`.
3. Do not add a separate SQLite driver dependency in Phase 1.
4. Update `docs/server.mmd` when implementing the module:

   ```mermaid
   config --> database
   ```

   Arrows point from a dependency to its consumer.

## Public API

`server/src/database/__init__.py` must declare a literal `__all__` and re-export only this interface:

```python
from collections.abc import ContextManager

from sqlalchemy import text
from sqlalchemy.engine import Engine
from sqlmodel import Field, SQLModel, Session, select

class DatabaseConfigurationError(ValueError): ...
class UnsupportedDatabaseBackendError(DatabaseConfigurationError): ...

def get_engine() -> Engine: ...
def session() -> ContextManager[Session]: ...
```

- `SQLModel`, `Field`, `select`, and `text` are re-exported so consumers use the module-root import as their supported database interface.
- `get_engine()` reads `DATABASE_URL`, validates it against the backends enabled for the current phase, and returns an engine. It creates no database file and executes no SQL. In Phase 1, non-SQLite URLs raise `UnsupportedDatabaseBackendError`; that restriction is explicitly temporary.
- Cache engines by their complete configured URL. In a normal process the configuration is fixed, so this provides one reusable engine; URL-keyed caching also permits isolated test URLs.
- Use private, dialect-specific engine options with SQL logging disabled. In Phase 1, SQLite engines receive `connect_args={"check_same_thread": False}`.
- `get_engine()` is also the database-module contract consumed by the Server migrations module. It exposes the SQLAlchemy engine Alembic needs to apply revision scripts without reading configuration or constructing a second engine.
- `session()` is a context manager. It opens a fresh `Session` against `get_engine()` and always closes it. Consumers call `commit()` for successful writes and `rollback()` after handling a failed transaction; the module must not commit implicitly.
- `DatabaseConfigurationError` is the common error for unusable database configuration. `UnsupportedDatabaseBackendError` identifies a backend not enabled in the current phase, rather than a permanent SQLite-only contract. Propagate SQLModel/SQLAlchemy execution errors unchanged.

Example consumer usage:

```python
from src.database import Field, SQLModel, select, session, text


class UserRecord(SQLModel, table=True):
    id: int | None = Field(default=None, primary_key=True)
    email: str


# The deployment/startup migration step has already applied the revision
# that creates this table.
with session() as database_session:
    user = UserRecord(email="person@example.com")
    database_session.add(user)
    database_session.commit()
    database_session.refresh(user)
    found = database_session.exec(
        select(UserRecord).where(UserRecord.email == user.email)
    ).one()
    email = database_session.exec(
        text("SELECT email FROM userrecord WHERE email = :email"),
        params={"email": user.email},
    ).one()
```

Raw SQL must always use `text()` with bound parameters; never interpolate values into SQL text.

## Schema lifecycle and migrations

This module performs no schema DDL. In particular, it must not call `SQLModel.metadata.create_all()` or expose a table-creation operation. The Server migrations module defined in `docs/prd-server-migrations.md` is the sole schema authority: it applies reviewed Alembic revision scripts before the server starts and records the current revision in Alembic's version table.

Consumer modules declare SQLModel tables for ORM use, but each table creation or alteration must be represented by a reviewed Alembic revision. The database module remains independent of those consumer models and their schema history.

## Layout

Start from `server/template` and create:

```text
server/src/database/
├── README.md
├── __init__.py
├── src/
│   ├── __init__.py
│   ├── _database.py
│   └── models.py
└── tests/
    ├── __init__.py
    └── test_database.py
```

Keep engine construction, URL validation, dialect options, and caching private. Keep the two public error classes in `models.py`. The module README must document the Phase 1 SQLite URL forms (`sqlite://` for in-memory and `sqlite:///./db.sqlite3` for a file), the planned PostgreSQL expansion, that schema creation is owned by the migrations module, session ownership, and commits.

## Test plan

All tests import only from `src.database` and declare every exercised export with `@pytest.mark.public_api`.

- Configure a temporary file URL and verify `get_engine()` is SQLite and returns the same engine for repeated calls with the same URL.
- Create a test-only table with test fixture DDL through the public `SQLModel.metadata` and `get_engine()` exports; this fixture is not a database-module operation.
- Exercise insert, commit, refresh, query with `select`, update, and delete inside `session()` contexts.
- Execute a parameterized `text()` query through a session and verify its result.
- Verify `session()` closes its session when its context exits.
- Verify a PostgreSQL URL raises `UnsupportedDatabaseBackendError` and is also a `DatabaseConfigurationError` while Phase 1 is in effect.
- Verify a missing `DATABASE_URL` preserves the configuration module's `ConfigurationValueError`.
- Ensure the public API coverage policy recognizes every export.

## Acceptance criteria

- A consumer can use an Alembic-created SQLModel table and perform CRUD through this module using a configured SQLite URL.
- A consumer can execute parameterized SQL through `text()` and a session.
- PostgreSQL URLs are rejected before an engine is created in Phase 1, and the error documents that PostgreSQL is a planned backend rather than an unsupported design direction.
- Importing `src.database` causes no database file creation, table creation, or connection attempt.
- The module has a consumer-oriented README, complete public-interface tests, and the updated Server module graph.
- `cd server && uv run pytest && uv run mypy .` and repository `pre-commit` pass.

## Follow-on: PostgreSQL enablement

Implement PostgreSQL as a subsequent, database-module-only change after Phase 1 is stable:

1. Add a maintained PostgreSQL DBAPI driver compatible with Python 3.14 and lock it with `uv`.
2. Accept and document `postgresql+<driver>://...` URLs, retaining the existing SQLite forms.
3. Add private PostgreSQL engine configuration without SQLite-only connection arguments.
4. Replace the Phase 1 PostgreSQL-rejection test with PostgreSQL integration tests covering engine creation, CRUD, parameterized SQL, transactions, and connection cleanup against a real disposable PostgreSQL instance. The migrations module separately validates schema upgrades on PostgreSQL.
5. Verify existing SQLite tests pass unchanged, proving consumer-facing operations remain portable.
6. Update the module README and this PRD's supported-backend language; do not change consumer imports or require consumer modules to branch by backend.

## References

- SQLModel, [Create a Table with SQLModel](https://sqlmodel.tiangolo.com/tutorial/create-db-and-table/): engine reuse, SQLite URL forms, and the distinction between table creation and migrations.
- SQLModel, [Session with FastAPI Dependency](https://sqlmodel.tiangolo.com/tutorial/fastapi/session-with-dependency/): one session per unit of work and SQLite `check_same_thread=False`.
