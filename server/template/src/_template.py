# SPDX-License-Identifier: AGPL-3.0-or-later

"""Private implementation for the Server module template."""

from .models import Template


def create(value: str) -> Template:
    """Create the placeholder public type.

    Replace this operation with the module's public behavior.
    """
    return _build_template(value)


def _build_template(value: str) -> Template:
    """Build the public result through a private implementation detail."""
    return Template(value=value)
