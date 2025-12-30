use crate::ast::Ident;
use phynix_core::Span;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ClosureUse {
    pub by_ref: bool,
    pub name: Ident,
    #[serde(skip)]
    pub span: Span,
}
