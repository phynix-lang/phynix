mod util;

use crate::util::{assert_kinds_eq, kinds_php_prefixed};
use phynix_lex::TokenKind;

#[test]
fn longest_operators() {
    let k = kinds_php_prefixed("=== !== ??= ?? ?-> -> << <<= >> >>= ** **= :: => ... ++ -- == != <= >= ~");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::StrictEq,
            TokenKind::StrictNe,
            TokenKind::NullCoalesceAssign,
            TokenKind::NullCoalesce,
            TokenKind::NullsafeArrow,
            TokenKind::Arrow,
            TokenKind::Shl,
            TokenKind::ShlEq,
            TokenKind::Shr,
            TokenKind::ShrEq,
            TokenKind::Pow,
            TokenKind::PowEq,
            TokenKind::ColCol,
            TokenKind::FatArrow,
            TokenKind::Ellipsis,
            TokenKind::PlusPlus,
            TokenKind::MinusMinus,
            TokenKind::EqEq,
            TokenKind::NotEq,
            TokenKind::Le,
            TokenKind::Ge,
            TokenKind::Tilde,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn spaceship_operator_is_tokenized() {
    let k = kinds_php_prefixed("1 <=> 2");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::Int,
            TokenKind::Spaceship,
            TokenKind::Int,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn and_and_operator_is_tokenized() {
    let k = kinds_php_prefixed("true && false");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::Ident,
            TokenKind::AndAnd,
            TokenKind::Ident,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn or_or_operator_is_tokenized() {
    let k = kinds_php_prefixed("true || false");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::Ident,
            TokenKind::OrOr,
            TokenKind::Ident,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn dot_eq_is_tokenized() {
    let k = kinds_php_prefixed("$a .= 1;");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::VarIdent,
            TokenKind::DotEq,
            TokenKind::Int,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn plus_eq_is_tokenized() {
    let k = kinds_php_prefixed("$a += 1;");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::VarIdent,
            TokenKind::PlusEq,
            TokenKind::Int,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn minus_eq_is_tokenized() {
    let k = kinds_php_prefixed("$a -= 1;");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::VarIdent,
            TokenKind::MinusEq,
            TokenKind::Int,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn mul_eq_is_tokenized() {
    let k = kinds_php_prefixed("$a *= 2;");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::VarIdent,
            TokenKind::MulEq,
            TokenKind::Int,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn div_eq_is_tokenized() {
    let k = kinds_php_prefixed("$a /= 2;");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::VarIdent,
            TokenKind::DivEq,
            TokenKind::Int,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn mod_eq_is_tokenized() {
    let k = kinds_php_prefixed("$a %= 2;");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::VarIdent,
            TokenKind::ModEq,
            TokenKind::Int,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn amp_eq_is_tokenized() {
    let k = kinds_php_prefixed("$a &= $b;");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::VarIdent,
            TokenKind::AmpEq,
            TokenKind::VarIdent,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn pipe_eq_is_tokenized() {
    let k = kinds_php_prefixed("$a |= $b;");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::VarIdent,
            TokenKind::PipeEq,
            TokenKind::VarIdent,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn caret_eq_is_tokenized() {
    let k = kinds_php_prefixed("$a ^= $b;");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::VarIdent,
            TokenKind::CaretEq,
            TokenKind::VarIdent,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ],
    );
}
