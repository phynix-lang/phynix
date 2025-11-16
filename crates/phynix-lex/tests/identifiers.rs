mod util;

use crate::util::{assert_kinds_eq, kinds};
use phynix_lex::TokenKind;

#[test]
fn idents_and_varidents() {
    let k = kinds("$über über _x");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::VarIdent,
            TokenKind::Ident,
            TokenKind::Ident,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn namespaces_and_sigils() {
    let k = kinds("$$foo ${bar} Foo\\Bar");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::Dollar,
            TokenKind::VarIdent,
            TokenKind::Dollar,
            TokenKind::LBrace,
            TokenKind::Ident,
            TokenKind::RBrace,
            TokenKind::Ident,
            TokenKind::Backslash,
            TokenKind::Ident,
            TokenKind::Eof,
        ],
    );
}
