use phynix_core::{Span, Spanned};
use serde::Serialize;

use super::Expr;

#[derive(Debug, Clone, Serialize)]
pub struct Ident {
    #[serde(skip)]
    pub span: Span,
}

impl Spanned for Ident {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Serialize)]
pub struct QualifiedName {
    pub absolute: bool,
    pub parts: Vec<Ident>,
    #[serde(skip)]
    pub span: Span,
}

impl Spanned for QualifiedName {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Serialize)]
pub enum SpecialClassName {
    SelfType,
    ParentType,
    StaticType,
}

#[derive(Debug, Serialize)]
pub enum ClassNameRef {
    Qualified(QualifiedName),
    Special(SpecialClassName),
    Dynamic(Box<Expr>),
}
