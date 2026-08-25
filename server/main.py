# SPDX-License-Identifier: AGPL-3.0-or-later


from src.api import app
import uvicorn


def main() -> None:
    """Run the Gradus HTTP server."""
    uvicorn.run(app, host="127.0.0.1", port=8000)


if __name__ == "__main__":
    main()
