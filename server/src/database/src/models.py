# SPDX-License-Identifier: AGPL-3.0-or-later

"""Public database error types."""


class DatabaseConfigurationError(ValueError):
    """Raised when the configured database URL cannot be used."""


class UnsupportedDatabaseBackendError(DatabaseConfigurationError):
    """Raised when a database backend is not enabled in the current phase."""
