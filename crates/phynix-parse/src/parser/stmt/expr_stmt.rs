use crate::ast::Stmt;
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub fn parse_empty_stmt(&mut self) -> Option<Stmt> {
        let semi_span = self.prev_span().unwrap_or(Span { start: 0, end: 0 });

        Some(Stmt::Noop { span: semi_span })
    }

    pub fn parse_expr_stmt(&mut self) -> Option<Stmt> {
        let start_pos = self.pos;

        let expr = if let Some(expr) = self.parse_expr() {
            expr
        } else {
            self.pos = start_pos;
            return None;
        };

        let end_span = if self.eat(TokenKind::Semicolon) {
            self.prev_span().unwrap()
        } else if self.at(TokenKind::PhpClose) || self.at(TokenKind::Eof) {
            let span = expr.span();
            Span {
                start: span.end,
                end: span.end,
            }
        } else {
            self.recover_to_any(&[
                TokenKind::Semicolon,
                TokenKind::RBrace,
                TokenKind::Dollar,
                TokenKind::Ident,
                TokenKind::AttrOpen,
            ]);

            expr.span()
        };

        let span = Span {
            start: expr.span().start,
            end: end_span.end,
        };

        Some(Stmt::ExprStmt { expr, span })
    }
}
