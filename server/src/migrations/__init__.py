# SPDX-License-Identifier: AGPL-3.0-or-later

"""Public interface for applying Server schema migrations."""

from .src._migrations import upgrade

__all__ = ["upgrade"]
