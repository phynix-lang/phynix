mod util;

use crate::util::{assert_kinds_eq, kinds_php_prefixed};
use phynix_lex::TokenKind;

#[test]
fn longest_operators() {
    let k = kinds_php_prefixed("=== !== ??= ?? ?-> -> << <<= >> >>= ** **= :: => ... ++ -- == != <= >=");
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
