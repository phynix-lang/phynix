use phynix_core::diagnostics::Diagnostic;
use phynix_core::{LanguageKind, Strictness};
use phynix_lex::lex;
use phynix_parse::ast::{Script, Stmt};
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

pub fn first_stmt(script: &Script) -> &Stmt {
    script.items.first().expect("expected at least 1 stmt")
}

pub fn assert_is_if(stmt: &Stmt) {
    match stmt {
        Stmt::If { .. } => {},
        other => panic!("expected Stmt::If, got: {other:#?}"),
    }
}

pub fn assert_else_has_n_items(stmt: &Stmt, n: usize) {
    match stmt {
        Stmt::If { else_block, .. } => {
            let b = else_block.as_ref().expect("expected else_block");
            assert_eq!(
                b.items.len(),
                n,
                "else_block items mismatch, else_block: {:#?}",
                b
            );
        },
        other => panic!("expected Stmt::If, got: {other:#?}"),
    }
}

pub fn assert_no_diags(diags: &[Diagnostic]) {
    assert!(
        diags.is_empty(),
        "expected no diagnostics, got:\n{:#?}",
        diags
    );
}
