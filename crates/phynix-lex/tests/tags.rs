mod util;

use crate::util::{assert_kinds_eq, kinds};
use phynix_core::{LanguageKind, Strictness};
use phynix_lex::{lex, TokenKind};

#[test]
fn echo_open_tag_is_tokenized() {
    let k = kinds("<?= 1 ?>");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::EchoOpen,
            TokenKind::Int,
            TokenKind::PhpClose,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn html_only_emits_single_html_chunk_to_eof() {
    let src = "hello <b>world</b>\nno php tags here";
    let k = kinds(src);

    assert_kinds_eq(&k, &[TokenKind::HtmlChunk, TokenKind::Eof]);
}

#[test]
fn html_only_chunk_span_covers_entire_input() {
    let src = "abc\nxyz";
    let out =
        lex(src, LanguageKind::PhpCompat, Strictness::Lenient).expect("lex ok");

    assert_eq!(out.tokens[0].kind, TokenKind::HtmlChunk);
    assert_eq!(out.tokens[0].span.start, 0);
    assert_eq!(out.tokens[0].span.end as usize, src.len());
}

#[test]
fn detects_php_open() {
    let k = kinds("<?php ");
    assert_kinds_eq(&k, &[TokenKind::PhpOpen, TokenKind::Eof]);
}

#[test]
fn detects_phx_open() {
    let k = kinds("<?phx ");
    assert_kinds_eq(&k, &[TokenKind::PhxOpen, TokenKind::Eof]);
}

#[test]
fn detects_phxt_open() {
    let k = kinds("<?phxt ");
    assert_kinds_eq(&k, &[TokenKind::PhxtOpen, TokenKind::Eof]);
}

#[test]
fn unknown_question_tag_stays_html() {
    let k = kinds("<?xD <?php");
    assert_kinds_eq(
        &k,
        &[TokenKind::HtmlChunk, TokenKind::PhpOpen, TokenKind::Eof],
    );
}

#[test]
fn phxt_open_tag_is_recognized() {
    let k = kinds("<?phxt echo 1 ?>");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::PhxtOpen,
            TokenKind::KwEcho,
            TokenKind::Int,
            TokenKind::PhpClose,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn phx_open_is_tokenized() {
    let k = kinds("<?phx $x ?>");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::PhxOpen,
            TokenKind::VarIdent,
            TokenKind::PhpClose,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn unknown_open_then_php_open_splits_html_chunk_then_php_open() {
    let k = kinds("<?xD\n<?php ");
    assert_kinds_eq(
        &k,
        &[TokenKind::HtmlChunk, TokenKind::PhpOpen, TokenKind::Eof],
    );
}

#[test]
fn html_then_echo_open_is_detected() {
    let k = kinds("x<?= ");
    assert_kinds_eq(
        &k,
        &[TokenKind::HtmlChunk, TokenKind::EchoOpen, TokenKind::Eof],
    );
}

#[test]
fn html_then_phxt_open_is_detected() {
    let k = kinds("x<?phxt ");
    assert_kinds_eq(
        &k,
        &[TokenKind::HtmlChunk, TokenKind::PhxtOpen, TokenKind::Eof],
    );
}
