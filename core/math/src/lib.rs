// SPDX-License-Identifier: AGPL-3.0-or-later

//! Typed arithmetic operations that preserve their operands and result.

mod api;
mod private;

pub use api::{Calculation, DivisionByZeroError, Operation, add, divide, multiply, subtract};
