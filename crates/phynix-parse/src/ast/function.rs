use crate::ast::{Expr, Ident, TypeRef};
use phynix_core::Span;

#[derive(Debug)]
pub struct Param {
    pub name: Ident,
    pub type_annotation: Option<TypeRef>,
    pub default: Option<Expr>,
    pub span: Span,
}
