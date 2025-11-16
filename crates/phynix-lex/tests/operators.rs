mod util;

use crate::util::{assert_kinds_eq, kinds};
use phynix_lex::TokenKind;

#[test]
fn longest_operators() {
    let k = kinds("=== !== ??= ?? ?-> -> << <<= >> >>= ** **= :: => ... ++ -- == != <= >=");
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
