# SPDX-License-Identifier: AGPL-3.0-or-later

"""Public interface for synchronous server database access."""

from sqlalchemy import text
from sqlmodel import Field, SQLModel, select

from .src._database import get_engine, session
from .src.models import DatabaseConfigurationError, UnsupportedDatabaseBackendError

__all__ = [
    "DatabaseConfigurationError",
    "Field",
    "SQLModel",
    "UnsupportedDatabaseBackendError",
    "get_engine",
    "select",
    "session",
    "text",
]
