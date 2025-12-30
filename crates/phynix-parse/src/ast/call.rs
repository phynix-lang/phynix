use crate::ast::{Expr, Ident};
use phynix_core::Span;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Arg {
    pub name: Option<Ident>,
    pub unpack: bool,
    pub expr: Expr,
    #[serde(skip)]
    pub span: Span,
}
