use crate::ast::{Expr, Ident, QualifiedName};
use phynix_core::{Span, Spanned};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AttributeGroup {
    pub attrs: Vec<Attribute>,
    #[serde(skip)]
    pub span: Span,
}

impl Spanned for AttributeGroup {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Serialize)]
pub struct Attribute {
    pub name: QualifiedName,
    pub args: Vec<AttributeArg>,
    #[serde(skip)]
    pub span: Span,
}

#[derive(Debug, Serialize)]
pub enum AttributeArg {
    Positional {
        expr: Expr,
        #[serde(skip)]
        span: Span,
    },
    Named {
        name: Ident,
        expr: Expr,
        #[serde(skip)]
        span: Span,
    },
}
