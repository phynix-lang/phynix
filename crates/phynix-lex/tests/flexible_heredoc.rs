use crate::util::{assert_kinds_eq, kinds_php_prefixed};
use phynix_core::token::TokenKind;

mod util;

#[test]
fn flexible_heredoc_allows_whitespace_before_terminator() {
    let src = "<<<LBL\nhello\nLBL ;\n";
    let k = kinds_php_prefixed(src);

    assert_kinds_eq(
        &k,
        &[TokenKind::StrDq, TokenKind::Semicolon, TokenKind::Eof],
    );
}

#[test]
fn flexible_heredoc_allows_various_terminators() {
    let cases = [
        ("<<<LBL\nhello\nLBL)", TokenKind::RParen),
        ("<<<LBL\nhello\nLBL]", TokenKind::RBracket),
        ("<<<LBL\nhello\nLBL}", TokenKind::RBrace),
        ("<<<LBL\nhello\nLBL,", TokenKind::Comma),
        ("<<<LBL\nhello\nLBL;", TokenKind::Semicolon),
    ];

    for (src, expected_term) in cases {
        let k = kinds_php_prefixed(src);
        assert_kinds_eq(&k, &[TokenKind::StrDq, expected_term, TokenKind::Eof]);
    }
}

#[test]
fn flexible_heredoc_allows_dot_after_label() {
    let src = "<<<LBL\nhello\nLBL.\n";
    let k = kinds_php_prefixed(src);

    assert_kinds_eq(&k, &[TokenKind::StrDq, TokenKind::Dot, TokenKind::Eof]);
}
