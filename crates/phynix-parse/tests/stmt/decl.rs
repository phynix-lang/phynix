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
fn class_members() {
    assert_script_snapshot!(
        "<?php
class Foo {
    public $a;
    protected $b;
    private $c;
    var $d;
    public static $e;
    public const F = 1;
    public function g() {}
}
"
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

#[test]
fn special_parameter_types() {
    assert_script_snapshot!("<?php class A { public function f(self $s, parent $p, static $st) {} }");
}
