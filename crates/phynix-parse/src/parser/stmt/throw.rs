use crate::ast::{Expr, Stmt};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_throw_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwThrow));

        let throw_token = self.bump();
        let start_pos = throw_token.span.start;
        let mut last_end = throw_token.span.end;

        let thrown_expr = if let Some(expr) = self.parse_expr() {
            last_end = expr.span().end;
            Some(expr)
        } else {
            self.error_here(
                ParseDiagnosticCode::ExpectedExpression,
                "expected expression after 'throw'",
            );
            None
        };

        let end_pos = if self.eat(TokenKind::Semicolon) {
            self.prev_span().unwrap().end
        } else if self.at(TokenKind::PhpClose) {
            self.current_span().start
        } else {
            self.error_and_recover(
                "expected ';' after throw",
                &[
                    TokenKind::Semicolon,
                    TokenKind::PhpClose,
                    TokenKind::RBrace,
                    TokenKind::Dollar,
                    TokenKind::Ident,
                    TokenKind::AttrOpen,
                ],
            );
            let _ = self.eat(TokenKind::Semicolon);
            self.prev_span().map(|s| s.end).unwrap_or(last_end)
        };

        let end_span = Span {
            start: end_pos,
            end: end_pos,
        };

        Some(Stmt::Throw {
            expr: thrown_expr.unwrap_or_else(|| Expr::Error {
                span: throw_token.span,
            }),
            span: Span {
                start: start_pos,
                end: end_span.end,
            },
        })
    }
}
