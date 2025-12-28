mod util;

use crate::util::{
    assert_kinds_eq, kinds_php_prefixed, lex_err, lex_err_php_prefixed,
};
use phynix_core::token::TokenKind;
use phynix_lex::LexError;

#[test]
fn braces_commas_colons() {
    let k =
        kinds_php_prefixed("{ } ( ) [ ] ; , : . + - * / % & | ^ @ ! = < > ?");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBracket,
            TokenKind::RBracket,
            TokenKind::Semicolon,
            TokenKind::Comma,
            TokenKind::Colon,
            TokenKind::Dot,
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
            TokenKind::Amp,
            TokenKind::Pipe,
            TokenKind::Caret,
            TokenKind::Silence,
            TokenKind::Bang,
            TokenKind::Eq,
            TokenKind::Lt,
            TokenKind::Gt,
            TokenKind::Question,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn unknown_byte_in_php_mode_errors_at_correct_offset() {
    let err = lex_err_php_prefixed("\u{0000}");
    match err {
        LexError::At(pos) => assert_eq!(pos, 7),
        other => panic!("expected LexError::At, got {other:?}"),
    }
}

#[test]
fn unknown_byte_in_php_mode_with_bom_accounts_for_prefix() {
    let err = lex_err("\u{FEFF}<?php \u{0000}");
    match err {
        LexError::At(pos) => assert_eq!(pos, 10),
        other => panic!("expected LexError::At, got {other:?}"),
    }
}
