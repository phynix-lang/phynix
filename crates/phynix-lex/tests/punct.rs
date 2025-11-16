mod util;

use crate::util::{assert_kinds_eq, kinds};
use phynix_lex::TokenKind;

#[test]
fn braces_commas_colons() {
    let k = kinds("{ } ( ) [ ] ; , : . + - * / % & | ^ @ ! = < > ?");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBracket,
            TokenKind::RBracket,
            TokenKind::Semicolon,
            TokenKind::Comma,
            TokenKind::Colon,
            TokenKind::Dot,
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
            TokenKind::Amp,
            TokenKind::Pipe,
            TokenKind::Caret,
            TokenKind::Silence,
            TokenKind::Bang,
            TokenKind::Eq,
            TokenKind::Lt,
            TokenKind::Gt,
            TokenKind::Question,
            TokenKind::Eof,
        ],
    );
}
