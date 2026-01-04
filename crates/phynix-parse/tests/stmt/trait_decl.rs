use crate::assert_script_snapshot;

#[test]
fn trait_simple() {
    assert_script_snapshot!("<?php trait Foo {}");
}
