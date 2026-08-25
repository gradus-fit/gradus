# SPDX-License-Identifier: AGPL-3.0-or-later

from collections.abc import Callable

from mypy.plugin import AnalyzeTypeContext, Plugin
from mypy.types import Type


class TypingPolicyPlugin(Plugin):
    def get_type_analyze_hook(
        self, fullname: str
    ) -> Callable[[AnalyzeTypeContext], Type] | None:
        if fullname == "builtins.object":
            return reject_object_type
        return None


def reject_object_type(context: AnalyzeTypeContext) -> Type:
    context.api.fail(
        "Use a precise type instead of the generic 'object' type.", context.context
    )
    return context.api.named_type("builtins.object", [])


def plugin(version: str) -> type[Plugin]:
    return TypingPolicyPlugin
