use crate::ast::{Ident, QualifiedName};
use phynix_core::Span;

#[derive(Debug, Copy, Clone)]
pub enum UseKind {
    Normal,
    Function,
    Const,
}

#[derive(Debug)]
pub struct UseImport {
    pub kind: UseKind,
    pub full_name: QualifiedName,
    pub alias: Option<Ident>,
    pub span: Span,
}
