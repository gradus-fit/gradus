# SPDX-License-Identifier: AGPL-3.0-or-later

"""Tests for the Server module template's public interface."""

import pytest

from .. import Template, create

MODULE_NAME = __name__.rsplit(".tests.", maxsplit=1)[0]


@pytest.mark.public_api(MODULE_NAME, "Template", "create")
def test_create_returns_the_public_template_type() -> None:
    """The public operation returns the documented public type."""
    template = create("example")

    assert isinstance(template, Template)
    assert template.value == "example"
