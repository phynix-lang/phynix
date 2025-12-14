use crate::util::{
    assert_else_has_n_items, assert_is_if, first_stmt, parse_ok,
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
