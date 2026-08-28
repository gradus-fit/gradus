# Configuration

Typed access to the server's allowlisted configuration. Other Server modules must use this module instead of reading `os.environ` directly.

## Public API

```python
from src.config import ConfigurationVariable, get_integer, get_string

host = get_string(ConfigurationVariable.SERVER_HOST)
port = get_integer(ConfigurationVariable.SERVER_PORT)
```

### Allowed variables

Only these environment variables may be read:

| Variable | Intended type | Description |
| --- | --- | --- |
| `DATABASE_URL` | string | Connection URL consumed by the database module. |
| `SERVER_HOST` | string | Address on which the HTTP server listens. |
| `SERVER_PORT` | integer | Port on which the HTTP server listens. |

Values can be exported in the process environment or stored in a `.env` file in the working directory (or one of its parents). Process environment values take precedence over `.env` values. `.env` values are read literally; references to other environment variables are not expanded.

### `ConfigurationVariable`

`StrEnum` naming the allowlisted variables. Pass a member to every getter; raw strings and variables not in this enum raise `UnknownConfigurationVariableError`.

### Getters

| Function | Return type | Behavior |
| --- | --- | --- |
| `get_string(variable)` | `str` | Returns the configured value unchanged. |
| `get_integer(variable)` | `int` | Parses the configured value as an integer. |
| `get_float(variable)` | `float` | Parses the configured value as a floating-point number. |
| `get_boolean(variable)` | `bool` | Parses case-insensitive `1`, `true`, `yes`, or `on` as true, and `0`, `false`, `no`, or `off` as false. |

All getters raise `ConfigurationValueError` when an allowlisted value is absent, cannot be parsed as the requested type, or the `.env` file cannot be read. `ConfigurationError` is their common base class.

## Example

```dotenv
# .env
SERVER_HOST=127.0.0.1
SERVER_PORT=8000
```

```python
from src.config import ConfigurationVariable, get_integer, get_string

assert get_string(ConfigurationVariable.SERVER_HOST) == "127.0.0.1"
assert get_integer(ConfigurationVariable.SERVER_PORT) == 8000
```
