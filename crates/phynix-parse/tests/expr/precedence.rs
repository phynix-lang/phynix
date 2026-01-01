use crate::assert_script_snapshot;

#[test]
fn test_precedence_null_coalesce_vs_binary() {
    // || has higher precedence than ??
    // $a || $b ?? $c should be ($a || $b) ?? $c
    assert_script_snapshot!("<?php $a || $b ?? $c;");
}

#[test]
fn test_precedence_ternary_vs_null_coalesce() {
    // ?? has higher precedence than ?:
    // $a ?? $b ? $c : $d should be ($a ?? $b) ? $c : $d
    assert_script_snapshot!("<?php $a ?? $b ? $c : $d;");
}

#[test]
fn test_precedence_assignment_vs_ternary() {
    // ?: has higher precedence than =
    // $a = $b ? $c : $d should be $a = ($b ? $c : $d)
    assert_script_snapshot!("<?php $a = $b ? $c : $d;");
}

#[test]
fn test_right_associativity_pow() {
    // ** is right-associative
    // $a ** $b ** $c should be $a ** ($b ** $c)
    assert_script_snapshot!("<?php $a ** $b ** $c;");
}

#[test]
fn test_right_associativity_null_coalesce() {
    // ?? is right-associative
    // $a ?? $b ?? $c should be $a ?? ($b ?? $c)
    assert_script_snapshot!("<?php $a ?? $b ?? $c;");
}

#[test]
fn test_non_associativity_ternary() {
    // ternary is non-associative
    // $a ? $b : $c ? $d : $e should produce an error
    let (_script, diags) = crate::util::parse("<?php $a ? $b : $c ? $d : $e;");
    insta::assert_yaml_snapshot!(diags);
}

#[test]
fn test_precedence_instanceof_vs_arithmetic() {
    // instanceof has higher precedence than *
    // $a * $b instanceof C should be $a * ($b instanceof C)
    assert_script_snapshot!("<?php $a * $b instanceof C;");
}

#[test]
fn test_precedence_comparison_non_associative() {
    // Comparisons are non-associative
    // $a < $b < $c is illegal
    let (_script, diags) = crate::util::parse("<?php $a < $b < $c;");
    insta::assert_yaml_snapshot!(diags);
}
