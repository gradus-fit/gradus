# SPDX-License-Identifier: AGPL-3.0-or-later

"""Tests for the API module's public interface."""

from httpx import ASGITransport, AsyncClient
import pytest

from .. import app

MODULE_NAME = __name__.rsplit(".tests.", maxsplit=1)[0]


@pytest.mark.public_api(MODULE_NAME, "app")
@pytest.mark.anyio
async def test_hello_endpoint_returns_greeting() -> None:
    """The application exposes a greeting at its root endpoint."""
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://testserver"
    ) as client:
        response = await client.get("/")

    assert response.status_code == 200
    assert response.text == '{"message":"Hello, World!"}'
