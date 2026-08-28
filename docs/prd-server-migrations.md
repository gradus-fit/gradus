# PRD: Alembic Server migrations module

## Status

Proposed. Implement after `docs/prd-server-config-database-url.md` and `docs/prd-server-database.md`. It creates the Server `migrations` module, which depends on `database`.

## Problem

Schema creation and change must be versioned, reviewed, reproducible, and separate from normal database access. `SQLModel.metadata.create_all()` cannot evolve an existing table safely and must not compete with the migration history.

## Goal

Provide a Server migrations module backed by Alembic. The normal server startup applies every reviewed migration before accepting traffic, and developers have a manual command to create an empty, reviewed migration revision.

## Architecture

```text
config --> database --> migrations
```

- `server/src/database` owns connection construction and CRUD/session access.
- `server/src/migrations` owns migration execution and the developer CLI.
- `server/alembic/` is application composition infrastructure, not a Server module. It contains the Alembic environment and the globally ordered revision history.
- Table-owning modules define their SQLModel tables but do not call `create_all()`. Every schema change is accompanied by a revision under `server/alembic/versions/`. This changes one feature module plus Server infrastructure, not two Server modules.

Create this layout from `server/template` for the module, and use Alembic's standard script layout for the application history:

```text
server/
├── alembic.ini
├── alembic/
│   ├── env.py
│   ├── script.py.mako
│   └── versions/
└── src/migrations/
    ├── README.md
    ├── __init__.py
    ├── __main__.py
    ├── src/
    │   ├── __init__.py
    │   └── _migrations.py
    └── tests/
        ├── __init__.py
        └── test_migrations.py
```

Update `docs/server.mmd` with `database --> migrations` when implementing.

## Dependencies

1. Add the current maintained `alembic` release compatible with Python 3.14 to `server/pyproject.toml` and regenerate `server/uv.lock`.
2. Alembic is the only schema-migration dependency. The runner passes an opened connection from `src.database.get_engine()` through Alembic's configuration attributes; `alembic/env.py` configures that supplied connection and never reads `DATABASE_URL` or creates another engine. `alembic.ini` contains no connection URL or credentials.
3. Do not add a database driver in this PRD. SQLite uses the driver already selected by the database module. PostgreSQL driver selection belongs to the database module's PostgreSQL follow-on.

## Public API and CLI

The module-root `__init__.py` has a literal `__all__` containing only:

```python
def upgrade() -> None: ...
```

`upgrade()` obtains `database.get_engine()`, opens an Alembic migration connection from that engine, and upgrades to `head`. It must:

- apply revisions in their declared order and let Alembic maintain its `alembic_version` table;
- be idempotent when the database is already at `head`;
- propagate configuration, engine, and Alembic failures so startup aborts rather than serving an incompatible schema; and
- never generate, modify, downgrade, or delete revision files.

The private CLI is the manual revision-creation interface:

```sh
cd server
uv run python -m src.migrations revision --message "create users"
```

It creates an empty Alembic revision under `alembic/versions/`. The developer writes explicit Alembic `op.*` upgrade and downgrade operations, reviews the resulting file, and commits it with the table-model change. Revision creation never runs automatically at startup.

The CLI also supports the explicit operational command:

```sh
cd server
uv run python -m src.migrations upgrade
```

The initial implementation deliberately does **not** use Alembic autogeneration. Autogeneration would require a global model-import/discovery contract across independent Server modules and can produce incomplete or backend-specific DDL. A future proposal may add it as a reviewed developer aid, never as an automatic migration author.

## Startup integration

`server/main.py` is the current Server composition root and must call `upgrade()` immediately before `uvicorn.run()`. A failed migration prevents the server from starting. The documented local startup command, `uv run python main.py`, therefore upgrades the configured SQLite database before serving requests.

Do not run migrations from import time, FastAPI route handlers, or normal database sessions. If a future deployment starts multiple PostgreSQL server processes concurrently, it must either run the CLI once as a deployment step or add database-specific migration serialization before retaining automatic startup upgrades.

## Revision rules

- Alembic revisions are the sole schema authority. Neither `database` nor consumer modules may call `SQLModel.metadata.create_all()` in production code.
- Every revision has deterministic, explicit `upgrade()` and `downgrade()` operations. A downgrade must be safe for the released data or explicitly raise `NotImplementedError` with a migration-specific explanation; it must never silently discard data.
- Use Alembic operations and backend-portable SQL types whenever possible. Isolate unavoidable SQLite- or PostgreSQL-specific DDL in the revision that requires it and document the backend limitation there.
- A data migration is a revision too. It must be bounded, resumable where needed, and safe to run exactly once under Alembic's version tracking.
- The initial baseline revision is empty and establishes Alembic version tracking. The first table-owning feature adds the first schema-changing revision.

## Test plan

All module tests import only from `src.migrations` and mark `upgrade` with `@pytest.mark.public_api`.

- Against a temporary configured SQLite URL, run `upgrade()` from an empty database and verify the baseline revision and `alembic_version` table are present.
- Run `upgrade()` twice and verify the second invocation is a no-op.
- Verify an unset `DATABASE_URL` preserves `ConfigurationValueError` from the configuration/database path.
- Verify an Alembic failure prevents successful `upgrade()` completion.
- Exercise the CLI revision command in an isolated temporary Alembic script location; verify it produces an empty revision file but does not modify the repository's real history.
- Once a schema-changing revision exists, run it against a temporary SQLite database and validate both upgrade and documented downgrade behavior.
- The database module's CRUD tests use test fixture DDL only; migrations integration tests validate production schema creation.

## Acceptance criteria

- The server's documented startup path upgrades the configured SQLite database to Alembic `head` before it serves traffic.
- Re-running startup against an up-to-date database is safe and does not create a second schema authority.
- Developers can create an empty, reviewed revision manually through the module CLI.
- Normal application code has no `create_all()` call or table-creation API.
- Migration code uses the engine from `src.database`, and `DATABASE_URL` remains read only by `config` through `database`.
- The module README, public-interface test coverage, `docs/server.mmd`, `uv run pytest`, `uv run mypy .`, and repository `pre-commit` pass when implemented.

## References

- Alembic, [Tutorial](https://alembic.sqlalchemy.org/en/latest/tutorial.html): revision scripts, `upgrade head`, environment configuration, and version tracking.
- SQLModel, [Create a Table with SQLModel](https://sqlmodel.tiangolo.com/tutorial/create-db-and-table/): distinction between `metadata.create_all()` and production migrations.
