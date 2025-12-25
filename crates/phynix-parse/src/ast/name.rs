use phynix_core::{Span, Spanned};

#[derive(Debug)]
pub struct Ident {
    pub span: Span,
}

impl Spanned for Ident {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug)]
pub struct QualifiedName {
    pub absolute: bool,
    pub parts: Vec<Ident>,
    pub span: Span,
}

impl Spanned for QualifiedName {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug)]
pub enum SpecialClassName {
    SelfType,
    ParentType,
    StaticType,
}

#[derive(Debug)]
pub enum ClassNameRef {
    Qualified(QualifiedName),
    Special(SpecialClassName),
}
