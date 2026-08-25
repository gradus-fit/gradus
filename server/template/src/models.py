# SPDX-License-Identifier: AGPL-3.0-or-later

"""Public typed data structures for the Server module template."""

from dataclasses import dataclass


@dataclass(frozen=True, slots=True, kw_only=True)
class Template:
    """Placeholder public type to replace with the module's domain type."""

    value: str
