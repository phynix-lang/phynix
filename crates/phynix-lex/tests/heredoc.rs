use crate::util::{assert_kinds_eq, kinds};
use phynix_lex::TokenKind;

mod util;

#[test]
fn heredoc_is_recognized_and_lexed_as_string_token() {
    let src = r#"<?php
$x = <<<TXT
hello
TXT
?>"#;

    let k = kinds(src);
    assert_kinds_eq(
        &k,
        &[
            TokenKind::PhpOpen,
            TokenKind::VarIdent,
            TokenKind::Eq,
            TokenKind::StrDq,
            TokenKind::PhpClose,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn triple_lt_not_a_valid_heredoc_rolls_back_and_lexes_as_shl_lt() {
    let src = r#"<?php
$x = <<<1
?>"#;

    let k = kinds(src);
    assert_kinds_eq(
        &k,
        &[
            TokenKind::PhpOpen,
            TokenKind::VarIdent,
            TokenKind::Eq,
            TokenKind::Shl,
            TokenKind::Lt,
            TokenKind::Int,
            TokenKind::PhpClose,
            TokenKind::Eof,
        ],
    );
}
