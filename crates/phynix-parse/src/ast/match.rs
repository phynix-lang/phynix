use crate::ast::Expr;
use phynix_core::Span;

#[derive(Debug)]
pub struct MatchArm {
    pub patterns: Vec<MatchPattern>,
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug)]
pub enum MatchPattern {
    Default { span: Span },
    Expr(Expr),
}
