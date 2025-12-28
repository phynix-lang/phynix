mod util;

use crate::util::{assert_kinds_eq, kinds_php_prefixed};
use phynix_core::token::TokenKind;

#[test]
fn ints_and_floats() {
    let k = kinds_php_prefixed("0 1 1_000 0xFF 0b1010 0o755 1.0 2e10");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::Int,
            TokenKind::Int,
            TokenKind::Int,
            TokenKind::Int,
            TokenKind::Int,
            TokenKind::Int,
            TokenKind::Float,
            TokenKind::Float,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn dot_leading_float_is_tokenized() {
    let k = kinds_php_prefixed(".4");
    assert_kinds_eq(&k, &[TokenKind::Float, TokenKind::Eof]);
}

#[test]
fn dot_leading_float_with_exponent_is_tokenized() {
    let k = kinds_php_prefixed(".4e2");
    assert_kinds_eq(&k, &[TokenKind::Float, TokenKind::Eof]);
}

#[test]
fn dot_leading_float_exponent_without_digits_rolls_back_to_dot_ident() {
    let k = kinds_php_prefixed(".4efoo");
    assert_kinds_eq(&k, &[TokenKind::Float, TokenKind::Ident, TokenKind::Eof]);
}

#[test]
fn dot_leading_float_exponent_allows_sign() {
    let k = kinds_php_prefixed(".4E-2");
    assert_kinds_eq(&k, &[TokenKind::Float, TokenKind::Eof]);
}

#[test]
fn int_with_exponent_plus_is_float() {
    let k = kinds_php_prefixed("1e+2");
    assert_kinds_eq(&k, &[TokenKind::Float, TokenKind::Eof]);
}

#[test]
fn int_with_exponent_minus_is_float() {
    let k = kinds_php_prefixed("1E-2");
    assert_kinds_eq(&k, &[TokenKind::Float, TokenKind::Eof]);
}

#[test]
fn dot_leading_float_with_exponent_plus_is_float() {
    let k = kinds_php_prefixed(".4e+2");
    assert_kinds_eq(&k, &[TokenKind::Float, TokenKind::Eof]);
}

#[test]
fn dot_leading_float_with_exponent_minus_is_float() {
    let k = kinds_php_prefixed(".4E-2");
    assert_kinds_eq(&k, &[TokenKind::Float, TokenKind::Eof]);
}

#[test]
fn exponent_without_digits_rolls_back_to_int_then_ident() {
    let k = kinds_php_prefixed("1e");
    assert_kinds_eq(&k, &[TokenKind::Int, TokenKind::Ident, TokenKind::Eof]);
}

#[test]
fn exponent_sign_without_digits_rolls_back_to_int_then_ident() {
    let k = kinds_php_prefixed("1e+x");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::Int,
            TokenKind::Ident,
            TokenKind::Plus,
            TokenKind::Ident,
            TokenKind::Eof,
        ],
    );
}
