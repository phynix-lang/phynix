use crate::ast::{Ident, QualifiedName};
use phynix_core::Span;
use serde::Serialize;

#[derive(Debug, Copy, Clone, Serialize)]
pub enum UseKind {
    Normal,
    Function,
    Const,
}

#[derive(Debug, Serialize)]
pub struct UseImport {
    pub kind: UseKind,
    pub full_name: QualifiedName,
    pub alias: Option<Ident>,
    #[serde(skip)]
    pub span: Span,
}
