use crate::util::{assert_kinds_eq_including_trivia, kinds};
use phynix_core::token::TokenKind;

mod util;

#[test]
fn html_chunk_before_open_tag_emits_htmlchunk_and_then_opens_php() {
    let src = "hello <?php $x ?>";

    let k = kinds(src);

    assert_kinds_eq_including_trivia(
        &k,
        &[
            TokenKind::HtmlChunk,
            TokenKind::PhpOpen,
            TokenKind::WS,
            TokenKind::VarIdent,
            TokenKind::WS,
            TokenKind::PhpClose,
            TokenKind::Eof,
        ],
    );
}
