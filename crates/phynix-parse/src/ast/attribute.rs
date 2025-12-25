use crate::ast::{Expr, Ident, QualifiedName};
use phynix_core::{Span, Spanned};

pub struct AttributeGroup {
    pub attrs: Vec<Attribute>,
    pub span: Span,
}

impl Spanned for AttributeGroup {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug)]
pub struct Attribute {
    pub name: QualifiedName,
    pub args: Vec<AttributeArg>,
    pub span: Span,
}

#[derive(Debug)]
pub enum AttributeArg {
    Positional { expr: Expr, span: Span },
    Named { name: Ident, expr: Expr, span: Span },
}
