use crate::ast::{Expr, Ident};
use phynix_core::Span;

#[derive(Debug)]
pub struct Arg {
    pub name: Option<Ident>,
    pub unpack: bool,
    pub expr: Expr,
    pub span: Span,
}
