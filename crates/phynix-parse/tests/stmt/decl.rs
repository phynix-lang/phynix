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

#[test]
fn class_simple() {
    assert_script_snapshot!("<?php class Foo {}");
}

#[test]
fn class_modifiers() {
    assert_script_snapshot!("<?php abstract final class Foo {}");
}

#[test]
fn class_extends_implements() {
    assert_script_snapshot!(
        "<?php class Foo extends Bar implements Baz, Qux {}"
    );
}

#[test]
fn interface_simple() {
    assert_script_snapshot!("<?php interface Foo {}");
}

#[test]
fn trait_simple() {
    assert_script_snapshot!("<?php trait Foo {}");
}

#[test]
fn enum_simple() {
    assert_script_snapshot!("<?php enum Foo {}");
}

#[test]
fn enum_backed() {
    assert_script_snapshot!("<?php enum Foo: string {}");
}
