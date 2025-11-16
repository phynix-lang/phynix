mod util;

use crate::util::{assert_kinds_eq, kinds};
use phynix_lex::TokenKind;

#[test]
fn dq_sq_bt_strings() {
    let k = kinds("\"a\\n\" 'x\\'y' `cmd`");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::StrDq,
            TokenKind::StrSq,
            TokenKind::StrBt,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn multiline_supported() {
    let k = kinds("\"a\nb\" `x\ny`");
    assert_kinds_eq(&k, &[TokenKind::StrDq, TokenKind::StrBt, TokenKind::Eof]);
}
