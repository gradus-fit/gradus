# API

HTTP application for the Gradus server.

## Public API

Import the ASGI application from `src.api`:

```python
from src.api import app
```

### `app`

A FastAPI application with a root `GET /` endpoint. It returns:

```json
{"message": "Hello, World!"}
```
