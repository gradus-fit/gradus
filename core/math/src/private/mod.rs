// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::api::{Calculation, Operation};

pub(crate) fn calculation(operation: Operation, left: f64, right: f64, result: f64) -> Calculation {
    Calculation {
        operation,
        left,
        right,
        result,
    }
}
