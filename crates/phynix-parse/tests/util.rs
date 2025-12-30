use phynix_core::diagnostics::Diagnostic;
use phynix_core::{LanguageKind, Strictness};
use phynix_lex::lex;
use phynix_parse::ast::Script;
use phynix_parse::parser::Parser;

pub fn parse_ok(src: &str) -> Script {
    let (script, diags) = parse(src);
    assert!(
        diags.is_empty(),
        "expected no diagnostics, got:\n{:#?}",
        diags
    );
    script
}

pub fn parse(src: &str) -> (Script, Vec<Diagnostic>) {
    let out =
        lex(src, LanguageKind::PhpCompat, Strictness::Lenient).expect("lex ok");
    Parser::new(
        src,
        &out.tokens,
        LanguageKind::PhpCompat,
        Strictness::Lenient,
    )
    .parse_script()
}

#[macro_export]
macro_rules! assert_script_snapshot {
    ($src:expr) => {
        let script = $crate::util::parse_ok($src);
        insta::assert_yaml_snapshot!(script);
    };
}

#[macro_export]
macro_rules! test_stmt {
    ($name:ident, $src:expr) => {
        #[test]
        fn $name() {
            $crate::assert_script_snapshot!($src);
        }
    };
}

#[macro_export]
macro_rules! test_expr {
    ($name:ident, $src:expr) => {
        #[test]
        fn $name() {
            $crate::assert_script_snapshot!(concat!("<?php ", $src, ";"));
        }
    };
}

#[macro_export]
macro_rules! test_error {
    ($name:ident, $src:expr) => {
        #[test]
        fn $name() {
            let (script, diags) = $crate::util::parse($src);
            insta::assert_yaml_snapshot!((script, diags));
        }
    };
}
