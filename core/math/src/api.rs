// SPDX-License-Identifier: AGPL-3.0-or-later

use std::error::Error;
use std::fmt;

use crate::private::calculation;

/// An arithmetic operation supported by this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// The operands and result of one arithmetic operation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Calculation {
    pub operation: Operation,
    pub left: f64,
    pub right: f64,
    pub result: f64,
}

/// Returned when division is attempted with a zero divisor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DivisionByZeroError;

impl fmt::Display for DivisionByZeroError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("cannot divide by zero")
    }
}

impl Error for DivisionByZeroError {}

/// Adds `right` to `left`.
pub fn add(left: f64, right: f64) -> Calculation {
    calculation(Operation::Add, left, right, left + right)
}

/// Subtracts `right` from `left`.
pub fn subtract(left: f64, right: f64) -> Calculation {
    calculation(Operation::Subtract, left, right, left - right)
}

/// Multiplies `left` by `right`.
pub fn multiply(left: f64, right: f64) -> Calculation {
    calculation(Operation::Multiply, left, right, left * right)
}

/// Divides `left` by `right`.
///
/// Returns [`DivisionByZeroError`] when `right` is zero.
pub fn divide(left: f64, right: f64) -> Result<Calculation, DivisionByZeroError> {
    if right == 0.0 {
        return Err(DivisionByZeroError);
    }

    Ok(calculation(Operation::Divide, left, right, left / right))
}
