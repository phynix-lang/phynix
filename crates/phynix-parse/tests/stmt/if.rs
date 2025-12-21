use crate::util::{
    assert_block_len, assert_echo_stmt, assert_if_stmt, assert_single_stmt,
    parse_ok,
};
use phynix_parse::ast::Stmt;

#[test]
fn else_body_accepts_single_statement() {
    let script = parse_ok(
        r#"<?php
        if (true) echo 1;
        else echo 2;
    "#,
    );

    let stmt = assert_single_stmt(&script);
    let (_cond, then_block, else_if_blocks, else_block) = assert_if_stmt(stmt);

    assert!(else_if_blocks.is_empty(), "{else_if_blocks:#?}");

    assert_block_len(then_block, 1, "then");
    let then_exprs = assert_echo_stmt(&then_block.items[0]);
    assert_eq!(then_exprs.len(), 1, "{then_exprs:#?}");

    let else_block = else_block.expect("expected else_block");
    assert_block_len(else_block, 1, "else");
    let else_exprs = assert_echo_stmt(&else_block.items[0]);
    assert_eq!(else_exprs.len(), 1, "{else_exprs:#?}");
}

#[test]
fn if_body_accepts_single_statement_without_braces() {
    let script = parse_ok(
        r#"<?php
        if (true) echo 1;
    "#,
    );

    let stmt = assert_single_stmt(&script);
    let (_cond, then_block, else_if_blocks, else_block) = assert_if_stmt(stmt);

    assert_block_len(then_block, 1, "then");
    let exprs = assert_echo_stmt(&then_block.items[0]);
    assert_eq!(exprs.len(), 1, "{exprs:#?}");

    assert!(else_if_blocks.is_empty(), "{else_if_blocks:#?}");
    assert!(else_block.is_none(), "{else_block:#?}");
}

#[test]
fn if_body_accepts_brace_block() {
    let script = parse_ok(
        r#"<?php
        if (true) { echo 1; echo 2; }
    "#,
    );

    let stmt = assert_single_stmt(&script);
    let (_cond, then_block, else_if_blocks, else_block) = assert_if_stmt(stmt);

    assert_block_len(then_block, 2, "then");
    assert_echo_stmt(&then_block.items[0]);
    assert_echo_stmt(&then_block.items[1]);

    assert!(else_if_blocks.is_empty(), "{else_if_blocks:#?}");
    assert!(else_block.is_none(), "{else_block:#?}");
}

#[test]
fn else_body_accepts_brace_block() {
    let script = parse_ok(
        r#"<?php
        if (true) echo 1;
        else { echo 2; echo 3; }
    "#,
    );

    let stmt = assert_single_stmt(&script);
    let (_cond, then_block, else_if_blocks, else_block) = assert_if_stmt(stmt);

    assert_block_len(then_block, 1, "then");
    assert_echo_stmt(&then_block.items[0]);

    let else_block = else_block.expect("expected else_block");
    assert_block_len(else_block, 2, "else");
    assert_echo_stmt(&else_block.items[0]);
    assert_echo_stmt(&else_block.items[1]);

    assert!(else_if_blocks.is_empty(), "{else_if_blocks:#?}");
}

#[test]
fn elseif_accepts_brace_block() {
    let script = parse_ok(
        r#"<?php
        if (false) echo 1;
        elseif (true) { echo 2; }
    "#,
    );

    let stmt = assert_single_stmt(&script);
    let (_cond, then_block, else_if_blocks, else_block) = assert_if_stmt(stmt);

    assert_block_len(then_block, 1, "then");
    assert_echo_stmt(&then_block.items[0]);

    assert_eq!(else_if_blocks.len(), 1, "{else_if_blocks:#?}");
    let (_elseif_cond, elseif_block) = &else_if_blocks[0];
    assert_block_len(elseif_block, 1, "elseif");
    assert_echo_stmt(&elseif_block.items[0]);

    assert!(else_block.is_none(), "{else_block:#?}");
}

#[test]
fn elseif_accepts_single_statement_without_braces() {
    let script = parse_ok(
        r#"<?php
        if (false) echo 1;
        elseif (true) echo 2;
    "#,
    );

    let stmt = assert_single_stmt(&script);
    let (_cond, _then_block, else_if_blocks, else_block) = assert_if_stmt(stmt);

    assert_eq!(else_if_blocks.len(), 1, "{else_if_blocks:#?}");
    let (_elseif_cond, elseif_block) = &else_if_blocks[0];
    assert_block_len(elseif_block, 1, "elseif");
    assert_echo_stmt(&elseif_block.items[0]);

    assert!(else_block.is_none(), "{else_block:#?}");
}

#[test]
fn multiple_elseifs_and_else_single_stmt() {
    let script = parse_ok(
        r#"<?php
        if (false) echo 1;
        elseif (false) echo 2;
        elseif (true) echo 3;
        else echo 4;
    "#,
    );

    let stmt = assert_single_stmt(&script);
    let (_cond, _then_block, else_if_blocks, else_block) = assert_if_stmt(stmt);

    assert_eq!(else_if_blocks.len(), 2, "{else_if_blocks:#?}");
    assert_block_len(&else_if_blocks[0].1, 1, "elseif #1");
    assert_block_len(&else_if_blocks[1].1, 1, "elseif #2");

    let else_block = else_block.expect("expected else_block");
    assert_block_len(else_block, 1, "else");
    assert_echo_stmt(&else_block.items[0]);
}

#[test]
fn else_if_is_parsed_as_nested_if_in_else_block() {
    // "else if" is valid PHP and is effectively nesting.
    let script = parse_ok(
        r#"<?php
        if (true) echo 1;
        else if (false) echo 2;
    "#,
    );

    let stmt = assert_single_stmt(&script);
    let (_cond, _then_block, else_if_blocks, else_block) = assert_if_stmt(stmt);

    assert!(else_if_blocks.is_empty(), "{else_if_blocks:#?}");

    let else_block = else_block.expect("expected else_block");
    assert_block_len(else_block, 1, "else");

    match &else_block.items[0] {
        Stmt::If { .. } => {},
        other => {
            panic!("expected nested Stmt::If in else block, got: {other:#?}")
        },
    }
}

#[test]
fn colon_syntax_if_without_else_is_parsed_and_endif_semicolon_optional() {
    let script = parse_ok(
        r#"<?php
        if (true):
            echo 1;
        endif
    "#,
    );

    let stmt = assert_single_stmt(&script);
    let (_cond, then_block, else_if_blocks, else_block) = assert_if_stmt(stmt);

    assert_block_len(then_block, 1, "then");
    assert_echo_stmt(&then_block.items[0]);

    assert!(else_if_blocks.is_empty(), "{else_if_blocks:#?}");
    assert!(else_block.is_none(), "{else_block:#?}");
}

#[test]
fn colon_syntax_if_with_else_and_semicolon_after_endif() {
    let script = parse_ok(
        r#"<?php
        if (false):
            echo 1;
        else:
            echo 2;
        endif;
    "#,
    );

    let stmt = assert_single_stmt(&script);
    let (_cond, then_block, else_if_blocks, else_block) = assert_if_stmt(stmt);

    assert_block_len(then_block, 1, "then");
    assert_echo_stmt(&then_block.items[0]);

    assert!(else_if_blocks.is_empty(), "{else_if_blocks:#?}");

    let else_block = else_block.expect("expected else_block");
    assert_block_len(else_block, 1, "else");
    assert_echo_stmt(&else_block.items[0]);
}

#[test]
fn colon_syntax_if_with_elseifs_and_else() {
    let script = parse_ok(
        r#"<?php
        if (false):
            echo 1;
        elseif (false):
            echo 2;
        elseif (true):
            echo 3;
        else:
            echo 4;
        endif;
    "#,
    );

    let stmt = assert_single_stmt(&script);
    let (_cond, then_block, else_if_blocks, else_block) = assert_if_stmt(stmt);

    assert_block_len(then_block, 1, "then");
    assert_echo_stmt(&then_block.items[0]);

    assert_eq!(else_if_blocks.len(), 2, "{else_if_blocks:#?}");
    assert_block_len(&else_if_blocks[0].1, 1, "elseif #1");
    assert_block_len(&else_if_blocks[1].1, 1, "elseif #2");

    let else_block = else_block.expect("expected else_block");
    assert_block_len(else_block, 1, "else");
    assert_echo_stmt(&else_block.items[0]);
}
