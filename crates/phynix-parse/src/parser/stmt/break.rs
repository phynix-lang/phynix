use crate::ast::{Expr, Stmt};
use crate::parser::Parser;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

impl<'src> Parser<'src> {
    pub(super) fn parse_break_stmt(&mut self) -> Option<Stmt> {
        self.parse_levelled_jump_stmt(TokenKind::KwBreak, |level, span| {
            Stmt::Break { level, span }
        })
    }

    pub(super) fn parse_continue_stmt(&mut self) -> Option<Stmt> {
        self.parse_levelled_jump_stmt(TokenKind::KwContinue, |level, span| {
            Stmt::Continue { level, span }
        })
    }

    fn parse_levelled_jump_stmt(
        &mut self,
        kw_kind: TokenKind,
        make: impl FnOnce(Option<Expr>, Span) -> Stmt,
    ) -> Option<Stmt> {
        debug_assert!(self.at(kw_kind));

        let kw = self.bump();
        let start = kw.span.start;
        let mut end = kw.span.end;

        let level = if self.at(TokenKind::Int) {
            if let Some(expr) = self.parse_int_literal() {
                end = expr.span().end;
                Some(expr)
            } else {
                None
            }
        } else {
            None
        };

        self.expect_or_err(TokenKind::Semicolon, &mut end);

        Some(make(level, Span { start, end }))
    }
}
