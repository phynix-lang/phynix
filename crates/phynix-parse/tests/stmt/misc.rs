use crate::assert_script_snapshot;

#[test]
fn return_simple() {
    assert_script_snapshot!("<?php return;");
}

#[test]
fn return_value() {
    assert_script_snapshot!("<?php return 1;");
}

#[test]
fn throw_simple() {
    assert_script_snapshot!("<?php throw $e;");
}

#[test]
fn break_simple() {
    assert_script_snapshot!("<?php break;");
}

#[test]
fn break_level() {
    assert_script_snapshot!("<?php break 2;");
}

#[test]
fn break_parenthesized_level() {
    assert_script_snapshot!("<?php break(2);");
}

#[test]
fn continue_parenthesized_level() {
    assert_script_snapshot!("<?php continue(2);");
}

#[test]
fn continue_simple() {
    assert_script_snapshot!("<?php continue;");
}

#[test]
fn global_simple() {
    assert_script_snapshot!("<?php global $a, $b;");
}

#[test]
fn static_var_simple() {
    assert_script_snapshot!("<?php static $a, $b = 1;");
}

#[test]
fn unset_simple() {
    assert_script_snapshot!("<?php unset($a, $b);");
}
