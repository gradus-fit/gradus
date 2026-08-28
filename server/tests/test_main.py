# SPDX-License-Identifier: AGPL-3.0-or-later

"""Tests for the Server composition root."""

from typing import Never

import pytest
from starlette.types import ASGIApp

import main as server_main


def test_main_upgrades_before_starting_uvicorn(monkeypatch: pytest.MonkeyPatch) -> None:
    """Schema compatibility is established before the server accepts traffic."""
    events: list[str] = []
    monkeypatch.setenv("SERVER_HOST", "127.0.0.1")
    monkeypatch.setenv("SERVER_PORT", "8000")

    def apply_migrations() -> None:
        events.append("upgrade")

    def serve(
        _application: ASGIApp, *, host: str, port: int, **_extra: Never
    ) -> None:
        assert host == "127.0.0.1"
        assert port == 8000
        events.append("serve")

    monkeypatch.setattr(server_main, "upgrade", apply_migrations)
    monkeypatch.setattr("main.uvicorn.run", serve)

    server_main.main()

    assert events == ["upgrade", "serve"]


def test_main_does_not_start_uvicorn_when_migration_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Migration failure prevents the process from accepting traffic."""
    server_started = False

    def fail_migrations() -> None:
        raise RuntimeError("migration failed")

    def serve(
        _application: ASGIApp, *, host: str, port: int, **_extra: Never
    ) -> None:
        nonlocal server_started
        server_started = True

    monkeypatch.setattr(server_main, "upgrade", fail_migrations)
    monkeypatch.setattr("main.uvicorn.run", serve)

    with pytest.raises(RuntimeError, match="migration failed"):
        server_main.main()

    assert not server_started
