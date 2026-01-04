use crate::assert_script_snapshot;

#[test]
fn function_simple() {
    assert_script_snapshot!("<?php function foo() {}");
}

#[test]
fn function_with_params() {
    assert_script_snapshot!("<?php function foo($a, $b = 1) {}");
}

#[test]
fn function_with_return_type() {
    assert_script_snapshot!("<?php function foo(): int {}");
}
