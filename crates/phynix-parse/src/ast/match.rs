use crate::ast::Expr;
use phynix_core::Span;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MatchArm {
    pub patterns: Vec<MatchPattern>,
    pub expr: Expr,
    #[serde(skip)]
    pub span: Span,
}

#[derive(Debug, Serialize)]
pub enum MatchPattern {
    Default {
        #[serde(skip)]
        span: Span,
    },
    Expr(Expr),
}
