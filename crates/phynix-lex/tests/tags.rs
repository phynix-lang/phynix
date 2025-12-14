mod util;

use crate::util::{assert_kinds_eq, kinds};
use phynix_core::{LanguageKind, Strictness};
use phynix_lex::{lex, TokenKind};

#[test]
fn echo_open_tag_is_tokenized() {
    let k = kinds("<?= 1 ?>");
    assert_kinds_eq(
        &k,
        &[
            TokenKind::EchoOpen,
            TokenKind::Int,
            TokenKind::PhpClose,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn html_only_emits_single_html_chunk_to_eof() {
    let src = "hello <b>world</b>\nno php tags here";
    let k = kinds(src);

    assert_kinds_eq(&k, &[TokenKind::HtmlChunk, TokenKind::Eof]);
}

#[test]
fn html_only_chunk_span_covers_entire_input() {
    let src = "abc\nxyz";
    let out =
        lex(src, LanguageKind::PhpCompat, Strictness::Lenient).expect("lex ok");

    assert_eq!(out.tokens[0].kind, TokenKind::HtmlChunk);
    assert_eq!(out.tokens[0].span.start, 0);
    assert_eq!(out.tokens[0].span.end as usize, src.len());
}
