mod util;

use crate::util::{assert_kinds_eq, kinds};
use phynix_lex::TokenKind;

#[test]
fn php_tags_and_echo_open() {
    let k = kinds("<?php <?= 1 ?>");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::PhpOpen,
            TokenKind::EchoOpen,
            TokenKind::Int,
            TokenKind::PhpClose,
            TokenKind::Eof,
        ],
    );
}
