# SPDX-License-Identifier: AGPL-3.0-or-later


from src.api import app
from src.config import ConfigurationVariable, get_integer, get_string
from src.migrations import upgrade
import uvicorn


def main() -> None:
    """Run the Gradus HTTP server."""
    upgrade()
    uvicorn.run(
        app,
        host=get_string(ConfigurationVariable.SERVER_HOST),
        port=get_integer(ConfigurationVariable.SERVER_PORT),
    )


if __name__ == "__main__":
    main()
