# Math

Typed arithmetic operations that return the operands, operation, and computed result for each calculation.

## Public API

Import the module interface from `src.math`:

```python
from src.math import Calculation, Operation, add, divide, multiply, subtract
```

### `Calculation`

Immutable dataclass returned by every arithmetic operation.

| Field | Type | Description |
| --- | --- | --- |
| `operation` | `Operation` | The operation that was performed. |
| `left` | `float` | The left operand. |
| `right` | `float` | The right operand. |
| `result` | `float` | The computed result. |

### `Operation`

Enum identifying the performed operation.

| Member | Value |
| --- | --- |
| `Operation.ADD` | `"add"` |
| `Operation.SUBTRACT` | `"subtract"` |
| `Operation.MULTIPLY` | `"multiply"` |
| `Operation.DIVIDE` | `"divide"` |

### `add`

```python
add(left: float, right: float) -> Calculation
```

Adds `right` to `left`.

### `subtract`

```python
subtract(left: float, right: float) -> Calculation
```

Subtracts `right` from `left`.

### `multiply`

```python
multiply(left: float, right: float) -> Calculation
```

Multiplies `left` by `right`.

### `divide`

```python
divide(left: float, right: float) -> Calculation
```

Divides `left` by `right`. Raises `ValueError` when `right` is zero.

## Example

```python
from src.math import Operation, divide

calculation = divide(9.0, 2.0)

assert calculation.operation is Operation.DIVIDE
assert calculation.left == 9.0
assert calculation.right == 2.0
assert calculation.result == 4.5
```
