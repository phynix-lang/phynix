use phynix_core::diagnostics::Diagnostic;
use phynix_core::{LanguageKind, Strictness};
use phynix_lex::lex;
use phynix_parse::ast::{Block, Expr, Script, Stmt};
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

pub fn assert_single_stmt(script: &Script) -> &Stmt {
    assert_eq!(
        script.items.len(),
        1,
        "expected exactly 1 top-level stmt, script: {:#?}",
        script
    );
    &script.items[0]
}

pub fn assert_if_stmt(
    stmt: &Stmt,
) -> (&Expr, &Block, &Vec<(Expr, Block)>, Option<&Block>) {
    match stmt {
        Stmt::If {
            cond,
            then_block,
            else_if_blocks,
            else_block,
            ..
        } => (cond, then_block, else_if_blocks, else_block.as_ref()),
        other => panic!("expected Stmt::If, got: {other:#?}"),
    }
}

pub fn assert_block_len(block: &Block, n: usize, label: &str) {
    assert_eq!(
        block.items.len(),
        n,
        "{label} block items mismatch (expected {n}), block: {:#?}",
        block
    );
}

pub fn assert_echo_stmt(stmt: &Stmt) -> &Vec<Expr> {
    match stmt {
        Stmt::Echo { exprs, .. } => exprs,
        other => panic!("expected Stmt::Echo, got: {other:#?}"),
    }
}
