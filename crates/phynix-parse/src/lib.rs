pub mod ast;
pub mod parser;

use crate::parser::Parser;
use ast::Script;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::Token;
use phynix_core::PhynixConfig;

pub struct ParseResult {
    pub ast: Script,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse(
    source: &str,
    tokens: &[Token],
    config: PhynixConfig,
) -> ParseResult {
    let parser = Parser::new(source, tokens, config);
    let (ast, diagnostics) = parser.parse_script();
    ParseResult { ast, diagnostics }
}
