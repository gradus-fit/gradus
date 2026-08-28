# PRD: Server configuration database URL

## Status

Proposed prerequisite. Implement this change in the existing `server/src/config` module before implementing the database module.

## Problem

A database module must obtain its connection string through the Server's allowlisted configuration interface. `ConfigurationVariable` currently has no database setting, so a new database module cannot meet that requirement without changing a second Server module.

## Goal

Allow Server modules to read one database connection string through the existing typed configuration API.

## Requirements

1. Add `DATABASE_URL = "DATABASE_URL"` to `ConfigurationVariable`.
2. Do not add a new getter. Callers use the existing `get_string(ConfigurationVariable.DATABASE_URL)`.
3. Preserve the existing precedence and parsing behavior: process environment before `.env`, literal `.env` values without interpolation, and `ConfigurationValueError` when the value is absent.
4. Update `server/src/config/README.md` to list `DATABASE_URL` as a string connection URL consumed by the database module.
5. Update `server/README.md` with a runnable SQLite example:

   ```dotenv
   DATABASE_URL=sqlite:///./db.sqlite3
   ```

6. Add a public-interface test that reads `DATABASE_URL` from the process environment. Mark it with `@pytest.mark.public_api(MODULE_NAME, "ConfigurationVariable", "get_string")`.

## Non-goals

- Do not parse, validate, normalize, or restrict the URL in `config`; database-backend policy belongs to the future database module.
- Do not add defaults or read arbitrary environment variables.
- Do not add SQLModel or any database driver dependency in this change.

## Acceptance criteria

- `get_string(ConfigurationVariable.DATABASE_URL)` returns the configured URL.
- An unset `DATABASE_URL` raises the existing `ConfigurationValueError`.
- The documented SQLite URL works with the subsequent database-module PRD.
- `cd server && uv run pytest && uv run mypy .` and repository `pre-commit` pass.
