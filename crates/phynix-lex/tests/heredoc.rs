use crate::util::{assert_kinds_eq, kinds, kinds_php_prefixed};
use phynix_core::token::TokenKind;

mod util;

#[test]
fn heredoc_is_recognized_and_lexed_as_string_token() {
    let src = r#"<?php
$x = <<<TXT
hello
TXT
?>"#;

    let k = kinds(src);
    assert_kinds_eq(
        &k,
        &[
            TokenKind::PhpOpen,
            TokenKind::VarIdent,
            TokenKind::Eq,
            TokenKind::StrDq,
            TokenKind::PhpClose,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn triple_lt_not_a_valid_heredoc_rolls_back_and_lexes_as_shl_lt() {
    let src = r#"<?php
$x = <<<1
?>"#;

    let k = kinds(src);
    assert_kinds_eq(
        &k,
        &[
            TokenKind::PhpOpen,
            TokenKind::VarIdent,
            TokenKind::Eq,
            TokenKind::Shl,
            TokenKind::Lt,
            TokenKind::Int,
            TokenKind::PhpClose,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn heredoc_allows_spaces_and_tabs_before_label() {
    let src = "<<< \tLABEL\nhello\nLABEL\n";

    let k = kinds_php_prefixed(src);
    assert_kinds_eq(&k, &[TokenKind::StrDq, TokenKind::Eof]);
}

#[test]
fn heredoc_with_quoted_label_consumes_closing_quote() {
    let src = "<<<\"LBL\"\nhello\nLBL\n";
    let k = kinds_php_prefixed(src);

    assert_kinds_eq(&k, &[TokenKind::StrDq, TokenKind::Eof]);
}

#[test]
fn heredoc_with_quoted_label_missing_closing_quote_rolls_back() {
    let src = "<<<\"LBL\nhello\nLBL\n";
    let k = kinds_php_prefixed(src);

    assert_kinds_eq(
        &k,
        &[
            TokenKind::Shl,
            TokenKind::Lt,
            TokenKind::StrDq,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn heredoc_header_ends_on_lf() {
    let src = "<<<LBL\nhello\nLBL\n";
    let k = kinds_php_prefixed(src);
    assert_kinds_eq(&k, &[TokenKind::StrDq, TokenKind::Eof]);
}

#[test]
fn heredoc_header_ends_on_crlf() {
    let src = "<<<LBL\r\nhello\r\nLBL\r\n";
    let k = kinds_php_prefixed(src);
    assert_kinds_eq(&k, &[TokenKind::StrDq, TokenKind::Eof]);
}

#[test]
fn heredoc_scan_breaks_when_body_is_immediately_eof() {
    let src = "<<<LBL\n";
    let k = kinds_php_prefixed(src);

    assert_kinds_eq(&k, &[TokenKind::StrDq, TokenKind::Eof]);
}

#[test]
fn heredoc_allows_indentation_before_closing_label() {
    let src = "<<<LBL\nhello\n \tLBL\n";
    let k = kinds_php_prefixed(src);

    assert_kinds_eq(&k, &[TokenKind::StrDq, TokenKind::Eof]);
}

#[test]
fn heredoc_scan_breaks_when_no_newline_exists_in_remaining_input() {
    let src = "<<<LBL\nhello";
    let k = kinds_php_prefixed(src);

    assert_kinds_eq(&k, &[TokenKind::StrDq, TokenKind::Eof]);
}

#[test]
fn heredoc_header_consumes_crlf() {
    let src = "<<<LBL\r\nx\r\nLBL\r\n";
    let k = kinds_php_prefixed(src);
    assert_kinds_eq(&k, &[TokenKind::StrDq, TokenKind::Eof]);
}

#[test]
fn heredoc_header_breaks_on_crlf() {
    let src = "<<<LBL\r\nbody\nLBL\n";
    let k = kinds_php_prefixed(src);
    assert_kinds_eq(&k, &[TokenKind::StrDq, TokenKind::Eof]);
}

#[test]
fn heredoc_header_allows_cr_without_lf() {
    let src = "<<<LBL\r";
    let k = kinds_php_prefixed(src);

    assert_kinds_eq(&k, &[TokenKind::StrDq, TokenKind::Eof]);
}
