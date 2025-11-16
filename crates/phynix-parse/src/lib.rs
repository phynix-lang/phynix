mod ast;
mod parser;

use ast::Script;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::{LanguageKind, Strictness};
use phynix_lex::Token;

pub struct ParseResult {
    pub ast: Script,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse(
    source: &str,
    tokens: &[Token],
    lang: LanguageKind,
    strictness: Strictness,
) -> ParseResult {
    let parser = parser::Parser::new(source, tokens, lang, strictness);
    let (ast, diagnostics) = parser.parse_script();
    ParseResult { ast, diagnostics }
}
