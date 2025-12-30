use crate::ast::Expr;
use phynix_core::Span;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ArrayItemExpr {
    pub key: Option<Expr>,
    pub value: Expr,
    pub unpack: bool,
    #[serde(skip)]
    pub span: Span,
}

#[derive(Debug, Serialize)]
pub struct ListItemExpr {
    pub key: Option<Expr>,
    pub value: Option<Expr>,
    #[serde(skip)]
    pub span: Span,
}
