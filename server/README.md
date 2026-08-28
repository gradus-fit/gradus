# Gradus Server

## Run locally

From this directory, configure the server through exported variables or a `.env` file:

```dotenv
DATABASE_URL=sqlite:///./db.sqlite3
SERVER_HOST=127.0.0.1
SERVER_PORT=8000
```

Then start the HTTP server:

```sh
uv run python main.py
```

Before accepting traffic, the command applies all reviewed Alembic migrations to `DATABASE_URL`. The server listens on the configured host and port. Visit `/` to receive the hello-world response.
