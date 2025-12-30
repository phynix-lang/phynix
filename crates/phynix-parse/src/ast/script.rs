use crate::ast::Stmt;
use phynix_core::{Span, Spanned};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Script {
    pub items: Vec<Stmt>,
    #[serde(skip)]
    pub span: Span,
}

impl Script {
    pub fn empty(span: Span) -> Self {
        Self {
            items: Vec::new(),
            span,
        }
    }

    pub fn push_stmt(&mut self, stmt: Stmt) {
        let stmt_end = stmt.span().end;
        self.items.push(stmt);
        self.span.end = stmt_end;
    }
}
