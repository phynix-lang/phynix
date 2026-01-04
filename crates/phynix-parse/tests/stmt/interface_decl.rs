use crate::assert_script_snapshot;

#[test]
fn interface_simple() {
    assert_script_snapshot!("<?php interface Foo {}");
}
