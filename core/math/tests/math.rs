// SPDX-License-Identifier: AGPL-3.0-or-later

use gradus_math::{Calculation, DivisionByZeroError, Operation, add, divide, multiply, subtract};

type OperationCase = (fn(f64, f64) -> Calculation, f64, f64, f64, Operation);

#[test]
fn operations_return_the_documented_calculation() {
    let cases: &[OperationCase] = &[
        (add, 2.0, 3.0, 5.0, Operation::Add),
        (subtract, 7.0, 3.0, 4.0, Operation::Subtract),
        (multiply, 2.5, 4.0, 10.0, Operation::Multiply),
    ];

    for (operation, left, right, expected_result, expected_operation) in cases {
        let calculation = operation(*left, *right);

        assert_eq!(calculation.operation, *expected_operation);
        assert_eq!(calculation.left, *left);
        assert_eq!(calculation.right, *right);
        assert_eq!(calculation.result, *expected_result);
    }

    let calculation = divide(9.0, 2.0).expect("a non-zero divisor succeeds");

    assert_eq!(calculation.operation, Operation::Divide);
    assert_eq!(calculation.left, 9.0);
    assert_eq!(calculation.right, 2.0);
    assert_eq!(calculation.result, 4.5);
}

#[test]
fn add_preserves_the_f64_operands_used_for_calculation() {
    let calculation = add(9_007_199_254_740_993.0, -9_007_199_254_740_992.0);

    assert_eq!(calculation.left + calculation.right, calculation.result);
}

#[test]
fn divide_rejects_a_zero_divisor() {
    assert_eq!(divide(1.0, 0.0), Err(DivisionByZeroError));
}

#[test]
fn division_by_zero_error_has_a_clear_message() {
    assert_eq!(DivisionByZeroError.to_string(), "cannot divide by zero");
}
