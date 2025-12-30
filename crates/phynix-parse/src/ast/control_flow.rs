use crate::ast::{Block, Expr, Ident, TypeRef};
use phynix_core::Span;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CatchClause {
    pub exception_types: Vec<TypeRef>,
    pub var: Option<Ident>,
    pub body: Block,
    #[serde(skip)]
    pub span: Span,
}

#[derive(Debug, Serialize)]
pub struct SwitchCase {
    pub condition: Option<Expr>,
    pub body: Block,
    #[serde(skip)]
    pub span: Span,
}
