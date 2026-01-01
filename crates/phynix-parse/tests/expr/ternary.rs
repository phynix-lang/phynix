use crate::util;

#[test]
fn test_elvis_chaining() {
    let src = "<?php $a ?: $b ?: $c;";
    let (_script, diags) = util::parse(src);
    assert!(
        diags.is_empty(),
        "Elvis chaining should be valid, but got: {:?}",
        diags
    );
}

#[test]
fn test_mixed_ternary_elvis_allowed() {
    let src = "<?php $a ?: $b ? $c : $d;";
    let (_script, diags) = util::parse(src);
    assert!(
        diags.is_empty(),
        "Elvis followed by ternary should be valid, but got: {:?}",
        diags
    );
}

#[test]
fn test_standard_ternary_non_associative_error() {
    let src = "<?php $a ? $b : $c ? $d : $e;";
    let (_script, diags) = util::parse(src);
    assert!(
        !diags.is_empty(),
        "Standard ternary chaining should be an error"
    );
    assert!(diags[0].message.contains("non-associative"));
}

#[test]
fn test_test2_php_fragment() {
    let src = "<?php
    $a['offsetClean'] <=> $b['offsetClean']
        ?: $a['order'] <=> $b['order']
        ?: ($a['type'] === SequenceMatcher::OP_DEL ? -1 : 1);";
    let (_script, diags) = util::parse(src);
    assert!(
        diags.is_empty(),
        "test2.php fragment should be valid, but got: {:?}",
        diags
    );
}

#[test]
fn test_parenthesized_ternary_allowed() {
    let src = "<?php $a ? $b : ($c ? $d : $e);";
    let (_script, diags) = util::parse(src);
    assert!(
        diags.is_empty(),
        "Parenthesized ternary should be valid, but got: {:?}",
        diags
    );
}
