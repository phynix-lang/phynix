use crate::ast::Stmt;
use phynix_core::{Span, Spanned};

#[derive(Debug)]
pub struct Script {
    pub items: Vec<Stmt>,
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
