use crate::assert_script_snapshot;

#[test]
fn int_literal() {
    assert_script_snapshot!("<?php 42;");
}

#[test]
fn float_literal() {
    assert_script_snapshot!("<?php 3.14;");
}

#[test]
fn string_literal() {
    assert_script_snapshot!("<?php 'hello';");
}

#[test]
fn bool_literal() {
    assert_script_snapshot!("<?php true;");
}

#[test]
fn null_literal() {
    assert_script_snapshot!("<?php null;");
}

#[test]
fn array_literal() {
    assert_script_snapshot!("<?php [1, 2, 3];");
}

#[test]
fn array_literal_with_keys() {
    assert_script_snapshot!("<?php ['a' => 1, 'b' => 2];");
}
