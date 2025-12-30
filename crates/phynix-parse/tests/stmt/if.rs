use crate::assert_script_snapshot;

#[test]
fn else_body_accepts_single_statement() {
    assert_script_snapshot!(
        r#"<?php
        if (true) echo 1;
        else echo 2;
    "#
    );
}

#[test]
fn if_body_accepts_single_statement_without_braces() {
    assert_script_snapshot!(
        r#"<?php
        if (true) echo 1;
    "#
    );
}

#[test]
fn if_body_accepts_brace_block() {
    assert_script_snapshot!(
        r#"<?php
        if (true) { echo 1; echo 2; }
    "#
    );
}

#[test]
fn else_body_accepts_brace_block() {
    assert_script_snapshot!(
        r#"<?php
        if (true) echo 1;
        else { echo 2; echo 3; }
    "#
    );
}

#[test]
fn elseif_accepts_brace_block() {
    assert_script_snapshot!(
        r#"<?php
        if (true) echo 1;
        elseif (false) { echo 2; }
    "#
    );
}

#[test]
fn elseif_accepts_single_statement_without_braces() {
    assert_script_snapshot!(
        r#"<?php
        if (true) echo 1;
        elseif (false) echo 2;
    "#
    );
}

#[test]
fn multiple_elseifs_and_else_single_stmt() {
    assert_script_snapshot!(
        r#"<?php
        if (false) echo 1;
        elseif (false) echo 2;
        elseif (true) echo 3;
        else echo 4;
    "#
    );
}

#[test]
fn else_if_is_parsed_as_nested_if_in_else_block() {
    assert_script_snapshot!(
        r#"<?php
        if (true) echo 1;
        else if (false) echo 2;
    "#
    );
}

#[test]
fn colon_syntax_if_without_else_is_parsed_and_endif_semicolon_optional() {
    assert_script_snapshot!(
        r#"<?php
        if (true):
            echo 1;
        endif
    "#
    );
}

#[test]
fn colon_syntax_if_with_else_and_semicolon_after_endif() {
    assert_script_snapshot!(
        r#"<?php
        if (false):
            echo 1;
        else:
            echo 2;
        endif;
    "#
    );
}

#[test]
fn colon_syntax_if_with_elseifs_and_else() {
    assert_script_snapshot!(
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
    "#
    );
}
