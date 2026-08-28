# Database

Synchronous SQLModel database access for Server modules. Import the public API from `src.database`; do not construct engines or read database configuration directly.

## Setup

Set `DATABASE_URL` to a SQLite SQLAlchemy URL:

```dotenv
DATABASE_URL=sqlite:///./db.sqlite3
```

Supported SQLite forms are `sqlite://` for an in-memory database and `sqlite:///./db.sqlite3` for a file.

## Public API

```python
from src.database import Field, SQLModel, select, session, text


class UserRecord(SQLModel, table=True):
    id: int | None = Field(default=None, primary_key=True)
    email: str


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

Use `text()` and bound parameters for raw SQL; never interpolate values into SQL text.

`session()` creates one fresh session and always closes it when its context exits. It does not commit automatically: call `commit()` for successful writes, and call `rollback()` after handling a failed transaction.

## Schema lifecycle

This module does not create or alter tables. Declare consumer-owned `SQLModel` tables for ORM use, then add reviewed Alembic revisions. The Server migrations module is the sole schema authority and applies those revisions before server startup.

`get_engine()` exposes the configured SQLAlchemy engine for the migrations module. It creates no tables, database files, or connections.

## Errors

`DatabaseConfigurationError` reports an unusable database URL. `UnsupportedDatabaseBackendError` is its subclass and indicates an unsupported database backend.
