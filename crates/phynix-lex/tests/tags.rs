mod util;

use crate::util::{assert_kinds_eq, kinds, kinds_phx, kinds_phxt};
use phynix_core::token::TokenKind;
use phynix_core::{LanguageKind, PhpVersion, PhynixConfig};
use phynix_lex::lex;

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
    let out = lex(
        src,
        PhynixConfig {
            target_php_version: PhpVersion::Php84,
            language: LanguageKind::Php,
        },
    )
    .expect("lex ok");

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
fn phx_code_starts_immediately() {
    let k = kinds_phx("$x = 1;");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::VarIdent,
            TokenKind::Eq,
            TokenKind::Int,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn phx_template_starts_as_html() {
    let k = kinds_phxt("hello <b>world</b>");
    assert_kinds_eq(&k, &[TokenKind::HtmlChunk, TokenKind::Eof]);
}

#[test]
fn phx_template_supports_php_tags_for_now() {
    let k = kinds_phxt("<?php $x ?>");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::PhpOpen,
            TokenKind::VarIdent,
            TokenKind::PhpClose,
            TokenKind::Eof,
        ],
    );
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
fn unknown_p_tag_stays_html() {
    let k = kinds("<?pa <?php");
    assert_kinds_eq(
        &k,
        &[TokenKind::HtmlChunk, TokenKind::PhpOpen, TokenKind::Eof],
    );
}

#[test]
fn trailing_less_than_is_html() {
    let k = kinds("abc <");
    assert_kinds_eq(&k, &[TokenKind::HtmlChunk, TokenKind::Eof]);
}
