mod util;

use crate::util::{assert_kinds_eq, kinds_php_prefixed};
use phynix_lex::TokenKind;

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
