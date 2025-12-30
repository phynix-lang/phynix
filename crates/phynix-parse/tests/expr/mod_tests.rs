use crate::assert_script_snapshot;

#[test]
fn binary_op() {
    assert_script_snapshot!("<?php 1 + 2 * 3;");
}

#[test]
fn assignment() {
    assert_script_snapshot!("<?php $a = 1;");
}

#[test]
fn compound_assignment() {
    assert_script_snapshot!("<?php $a += 1;");
}

#[test]
fn function_call() {
    assert_script_snapshot!("<?php foo(1, 2);");
}

#[test]
fn method_call() {
    assert_script_snapshot!("<?php $obj->foo(1);");
}

#[test]
fn static_call() {
    assert_script_snapshot!("<?php Foo::bar(1);");
}

#[test]
fn match_expr() {
    assert_script_snapshot!(
        "<?php match ($a) { 1 => 'one', default => 'other' };"
    );
}

#[test]
fn ternary() {
    assert_script_snapshot!("<?php $a ? $b : $c;");
}

#[test]
fn null_coalesce() {
    assert_script_snapshot!("<?php $a ?? $b;");
}
