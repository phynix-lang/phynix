mod util;

use crate::util::{
    assert_kinds_eq, assert_kinds_eq_including_trivia, kinds,
    kinds_php_prefixed, lex_err_php_prefixed,
};
use phynix_lex::TokenKind;

#[test]
fn attr_open_beats_hash_comment() {
    let k = kinds_php_prefixed("#[Route]\n");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::AttrOpen,
            TokenKind::Ident,
            TokenKind::RBracket,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn hash_comment_is_skipped() {
    let k = kinds_php_prefixed("# hello\n42");
    assert_kinds_eq_including_trivia(
        &k,
        &[
            TokenKind::HashComment,
            TokenKind::WS,
            TokenKind::Int,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn docblock_kept_block_comment_skipped() {
    let k = kinds_php_prefixed("/** @var int */ $x; /* trailing */");
    assert_kinds_eq_including_trivia(
        &k,
        &[
            TokenKind::Docblock,
            TokenKind::WS,
            TokenKind::VarIdent,
            TokenKind::Semicolon,
            TokenKind::WS,
            TokenKind::BlockComment,
            TokenKind::Eof,
        ],
    )
}

#[test]
fn unterminated_block_comment_errors() {
    let e = lex_err_php_prefixed("/* no end");
    let msg = format!("{}", e);
    assert!(msg.contains("Unterminated block comment"), "got: {msg}");
}

#[test]
fn line_comment_stops_at_newline() {
    let k = kinds_php_prefixed("// hello\n42");
    assert_kinds_eq_including_trivia(
        &k,
        &[
            TokenKind::LineComment,
            TokenKind::WS,
            TokenKind::Int,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn line_comment_stops_before_php_close() {
    let k = kinds("<?php // hello ?>42");
    assert_kinds_eq_including_trivia(
        &k,
        &[
            TokenKind::PhpOpen,
            TokenKind::WS,
            TokenKind::LineComment,
            TokenKind::PhpClose,
            TokenKind::HtmlChunk,
            TokenKind::Eof,
        ],
    );
}
