use crate::assert_script_snapshot;

#[test]
fn echo_simple() {
    assert_script_snapshot!("<?php echo 1;");
}

#[test]
fn echo_multiple() {
    assert_script_snapshot!("<?php echo 1, 2, 3;");
}

#[test]
fn echo_no_semicolon_at_end_of_file() {
    assert_script_snapshot!("<?php echo 1");
}

#[test]
fn echo_no_semicolon_before_close_tag() {
    assert_script_snapshot!("<?php echo 1 ?>");
}
