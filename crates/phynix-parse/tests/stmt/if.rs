use crate::util::{
    assert_else_has_n_items, assert_is_if, assert_no_diags, first_stmt, parse,
    parse_ok,
};

#[test]
fn else_body_accepts_single_statement() {
    let src = r#"<?php
        if (true) echo 1;
        else echo 2;
    "#;

    let script = parse_ok(src);
    let stmt = first_stmt(&script);
    assert_is_if(stmt);

    assert_else_has_n_items(stmt, 1);
}

#[test]
fn else_body_accepts_nested_if_without_braces() {
    let src = r#"<?php
        if (true) echo 1;
        else if (false) echo 2;
    "#;

    let script = parse_ok(src);
    let stmt = first_stmt(&script);
    assert_is_if(stmt);

    assert_else_has_n_items(stmt, 1);
}

#[test]
fn if_colon_with_php_bounce_does_not_consume_outer_endif() {
    let src = r#"<?php
if (true) { ?>
    <?php
    if (true) :
        if (true) {
            ?>
            <?php
        }
    endif;
    ?>
    <?php
}
?>"#;

    let (_script, diags) = parse(src);
    assert_no_diags(&diags);
}

#[test]
fn inner_endif_does_not_end_outer_block() {
    let src = r#"<?php
if (true) { ?>
    <?php
    if (true) :
        echo 1;
    endif;
    echo 2;
    ?>
    <?php
}
?>"#;

    let (script, diags) = parse(src);
    assert_no_diags(&diags);

    assert_eq!(script.items.len(), 1, "script: {:#?}", script);
}
