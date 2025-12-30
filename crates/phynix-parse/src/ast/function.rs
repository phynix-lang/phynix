use crate::ast::{Expr, Ident, TypeRef};
use phynix_core::Span;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Param {
    pub name: Ident,
    pub type_annotation: Option<TypeRef>,
    pub default: Option<Expr>,
    #[serde(skip)]
    pub span: Span,
}
