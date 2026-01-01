use crate::util::assert_kinds_eq;
use phynix_core::token::TokenKind;
use phynix_core::{LanguageKind, PhpVersion, PhynixConfig};
use phynix_lex::lex;

mod util;

#[test]
fn bom_is_skipped_before_first_token_kind() {
    let src = "\u{FEFF}<?php echo 1 ?>";
    let out = lex(
        src,
        PhynixConfig {
            target_php_version: PhpVersion::Php84,
            language: LanguageKind::Php,
        },
    )
    .expect("lex ok");
    let kinds: Vec<_> = out.tokens.iter().map(|t| t.kind).collect();

    assert_kinds_eq(
        &kinds,
        &[
            TokenKind::PhpOpen,
            TokenKind::KwEcho,
            TokenKind::Int,
            TokenKind::PhpClose,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn bom_does_not_shift_spans_they_still_match_original_input() {
    let src = "\u{FEFF}<?php echo 1 ?>";
    let out = lex(
        src,
        PhynixConfig {
            target_php_version: PhpVersion::Php84,
            language: LanguageKind::Php,
        },
    )
    .expect("lex ok");

    let php_open = out
        .tokens
        .iter()
        .find(|t| t.kind == TokenKind::PhpOpen)
        .unwrap();
    assert_eq!(php_open.span.start, 3);
    assert_eq!(php_open.span.end, 8); // "<?php" is 5 bytes, starts after BOM

    let echo_kw = out
        .tokens
        .iter()
        .find(|t| t.kind == TokenKind::KwEcho)
        .unwrap();
    assert_eq!(
        &src[echo_kw.span.start as usize..echo_kw.span.end as usize],
        "echo"
    );
}

#[test]
fn no_bom_spans_start_at_zero() {
    let src = "<?php echo 1 ?>";
    let out = lex(
        src,
        PhynixConfig {
            target_php_version: PhpVersion::Php84,
            language: LanguageKind::Php,
        },
    )
    .expect("lex ok");

    let php_open = out
        .tokens
        .iter()
        .find(|t| t.kind == TokenKind::PhpOpen)
        .unwrap();
    assert_eq!(php_open.span.start, 0);
    assert_eq!(php_open.span.end, 5);
}
