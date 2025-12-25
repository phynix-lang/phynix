use crate::ast::Ident;
use phynix_core::Span;

#[derive(Debug)]
pub struct ClosureUse {
    pub by_ref: bool,
    pub name: Ident,
    pub span: Span,
}
