# SPDX-License-Identifier: AGPL-3.0-or-later

"""Public interface for a Server module template."""

from .src._template import create
from .src.models import Template

__all__ = ["Template", "create"]
