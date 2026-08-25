# SPDX-License-Identifier: AGPL-3.0-or-later

"""FastAPI application implementation."""

from fastapi import FastAPI

app = FastAPI()


@app.get("/")  # type: ignore[misc] # FastAPI route decorators currently expose Any.
async def hello() -> dict[str, str]:
    """Return a greeting to confirm the server is running."""
    return {"message": "Hello, World!"}
