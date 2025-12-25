use crate::ast::{Block, Expr, Ident, TypeRef};
use phynix_core::Span;

#[derive(Debug)]
pub struct CatchClause {
    pub exception_types: Vec<TypeRef>,
    pub var: Option<Ident>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug)]
pub struct SwitchCase {
    pub condition: Option<Expr>,
    pub body: Block,
    pub span: Span,
}
