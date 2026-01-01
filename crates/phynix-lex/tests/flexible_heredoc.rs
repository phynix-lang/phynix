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
fn flexible_heredoc_does_not_allow_dot_after_label() {
    // According to the issue, dot should probably be excluded to avoid breaking valid body text.
    // The current code has it, but the issue says: "Prevent False Positives: Do not terminate if the character immediately following the label is part of the text (e.g., . for concatenation should be handled carefully or excluded to avoid breaking valid body text that matches the label name)."
    let src = "<<<LBL\nhello\nLBL.\n";
    let k = kinds_php_prefixed(src);

    // If it's NOT a closing label, it should all be one StrDq until it finds a proper LBL
    // But since there is no other LBL, it will eventually EOF.
    assert_kinds_eq(&k, &[TokenKind::StrDq, TokenKind::Eof]);
}
