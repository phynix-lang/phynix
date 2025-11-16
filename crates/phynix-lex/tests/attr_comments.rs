mod util;

use crate::util::{assert_kinds_eq, assert_kinds_eq_including_trivia, kinds};
use phynix_lex::TokenKind;

#[test]
fn attr_open_beats_hash_comment() {
    let k = kinds("#[Route]\n");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::AttrOpen,
            TokenKind::Ident,
            TokenKind::RBracket,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn hash_comment_is_skipped() {
    let k = kinds("# hello\n42");
    assert_kinds_eq_including_trivia(
        &k,
        &[
            TokenKind::HashComment,
            TokenKind::WS,
            TokenKind::Int,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn docblock_kept_block_comment_skipped() {
    let k = kinds("/** @var int */ $x; /* trailing */");
    assert_kinds_eq_including_trivia(
        &k,
        &[
            TokenKind::Docblock,
            TokenKind::WS,
            TokenKind::VarIdent,
            TokenKind::Semicolon,
            TokenKind::WS,
            TokenKind::BlockComment,
            TokenKind::Eof,
        ],
    )
}
