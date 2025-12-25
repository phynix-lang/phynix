use crate::ast::Expr;
use phynix_core::Span;

#[derive(Debug)]
pub struct ArrayItemExpr {
    pub key: Option<Expr>,
    pub value: Expr,
    pub unpack: bool,
    pub span: Span,
}

#[derive(Debug)]
pub struct ListItemExpr {
    pub key: Option<Expr>,
    pub value: Option<Expr>,
    pub span: Span,
}
