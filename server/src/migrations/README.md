# Migrations

Applies the Server's reviewed Alembic schema history. Import only from `src.migrations`.

## Apply migrations

`upgrade()` applies revisions through the engine supplied by `src.database` and raises any configuration, connection, or Alembic error. It is safe to call when the database is already at `head`.

```python
from src.migrations import upgrade

upgrade()
```

The Server composition root calls this operation before it starts accepting traffic. Do not create or alter production tables with `SQLModel.metadata.create_all()`.

## Create a revision

From `server/`, create an empty revision for a reviewed schema change:

```sh
uv run python -m src.migrations revision --message "create users"
```

Write explicit Alembic `op.*` operations in the generated revision, review it, and commit it with the table-model change. Revision creation never runs automatically.

To apply the history explicitly:

```sh
uv run python -m src.migrations upgrade
```
