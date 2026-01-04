use crate::assert_script_snapshot;

#[test]
fn enum_simple() {
    assert_script_snapshot!("<?php enum Foo {}");
}

#[test]
fn enum_backed() {
    assert_script_snapshot!("<?php enum Foo: string {}");
}
