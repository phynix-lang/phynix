use phynix_core::token::{Token, TokenKind};
use phynix_core::{LanguageKind, Strictness};
use phynix_lex::{lex, LexError};

fn lex_ok(src: &str) -> Vec<Token> {
    let out =
        lex(src, LanguageKind::PhpCompat, Strictness::Lenient).expect("lex ok");
    out.tokens
}

pub fn lex_err(src: &str) -> LexError {
    lex(src, LanguageKind::PhpCompat, Strictness::Lenient)
        .err()
        .expect("expected err")
}

pub fn lex_err_php_prefixed(src: &str) -> LexError {
    let prefix = "<?php ";
    let prefixed = format!("{}{}", prefix, src);
    lex(&prefixed, LanguageKind::PhpCompat, Strictness::Lenient)
        .err()
        .expect("expected err")
}

pub fn kinds(src: &str) -> Vec<TokenKind> {
    lex_ok(src).into_iter().map(|token| token.kind).collect()
}

pub fn kinds_php_prefixed(src: &str) -> Vec<TokenKind> {
    let prefix = "<?php ";
    let prefixed = format!("{}{}", prefix, src);
    let prefix_len = prefix.len();
    lex_ok(&prefixed)
        .into_iter()
        .filter(|token| (token.span.start as usize) >= prefix_len)
        .map(|token| token.kind)
        .collect()
}

fn texts(src: &str) -> Vec<String> {
    let tokens = lex_ok(src);
    tokens
        .iter()
        .map(|token| {
            src[token.span.start as usize..token.span.end as usize].to_string()
        })
        .collect()
}

pub fn assert_kinds_eq(actual: &[TokenKind], expected: &[TokenKind]) {
    assert_kinds_eq_inner(actual, expected, false);
}

pub fn assert_kinds_eq_including_trivia(
    actual: &[TokenKind],
    expected: &[TokenKind],
) {
    assert_kinds_eq_inner(actual, expected, true);
}

fn assert_kinds_eq_inner(
    actual: &[TokenKind],
    expected: &[TokenKind],
    include_trivia: bool,
) {
    let filtered: Vec<_> = if include_trivia {
        actual.to_vec()
    } else {
        actual
            .iter()
            .filter(|k| !(*k).is_trivia())
            .cloned()
            .collect()
    };

    assert_eq!(
        filtered.len(),
        expected.len(),
        "length mismatch:\n  actual:   {:?}\n  expected: {:?}",
        filtered,
        expected
    );

    for (i, (a, e)) in filtered.iter().zip(expected.iter()).enumerate() {
        assert_eq!(
            std::mem::discriminant(a),
            std::mem::discriminant(e),
            "kind[{}] differs: got {:?}, expected {:?}",
            i,
            a,
            e
        );
    }
}
