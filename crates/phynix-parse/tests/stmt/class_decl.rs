use crate::assert_script_snapshot;

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
fn special_parameter_types() {
    assert_script_snapshot!("<?php class A { public function f(self $s, parent $p, static $st) {} }");
}

#[test]
fn class_property_asymmetric_visibility_implied_public() {
    assert_script_snapshot!(
        "<?php
class AsymVisibility {
    private(set) $x = 'hello';
}
"
    );
}

#[test]
fn class_property_asymmetric_visibility_explicit_public() {
    assert_script_snapshot!(
        "<?php
class AsymVisibility {
    public private(set) $x = 'hello';
}
"
    );
}

#[test]
fn class_property_hooks_block_get_set() {
    assert_script_snapshot!(
        "<?php
class Hooks {
    public string $x {
        get { return $this->x; }
        set(string $value) { $this->x = $value; }
    }
}
"
    );
}

#[test]
fn class_property_hooks_short_get() {
    assert_script_snapshot!(
        "<?php
class Hooks {
    public string $x {
        get => $this->x;
    }
}
"
    );
}
