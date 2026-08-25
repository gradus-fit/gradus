# Math

Typed arithmetic operations that return the operands, operation, and computed result for each calculation.

## Public API

Import the crate interface from `gradus_math`:

```rust
use gradus_math::{add, divide, multiply, subtract, Calculation, DivisionByZeroError, Operation};
```

### `Calculation`

A result returned by every arithmetic operation.

| Field | Type | Description |
| --- | --- | --- |
| `operation` | `Operation` | The operation that was performed. |
| `left` | `f64` | The left operand. |
| `right` | `f64` | The right operand. |
| `result` | `f64` | The computed result. |

### `Operation`

Enum identifying the performed operation:

- `Operation::Add`
- `Operation::Subtract`
- `Operation::Multiply`
- `Operation::Divide`

### Operations

```rust
fn add(left: f64, right: f64) -> Calculation
fn subtract(left: f64, right: f64) -> Calculation
fn multiply(left: f64, right: f64) -> Calculation
fn divide(left: f64, right: f64) -> Result<Calculation, DivisionByZeroError>
```

`divide` returns `DivisionByZeroError` when `right` is zero.

## Verification

From `core/`, run the required coverage check:

```sh
cargo coverage
```

This requires [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov), installed with:

```sh
cargo install cargo-llvm-cov --locked
```

The command fails unless every line and function in the workspace is covered.

## Example

```rust
use gradus_math::{divide, Operation};

let calculation = divide(9.0, 2.0)?;

assert_eq!(calculation.operation, Operation::Divide);
assert_eq!(calculation.left, 9.0);
assert_eq!(calculation.right, 2.0);
assert_eq!(calculation.result, 4.5);

# Ok::<(), gradus_math::DivisionByZeroError>(())
```
