use crate::ast::Stmt;
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

impl<'src> Parser<'src> {
    pub fn parse_empty_stmt(&mut self, semi_span: Span) -> Stmt {
        Stmt::Noop { span: semi_span }
    }

    pub fn parse_expr_stmt(&mut self) -> Option<Stmt> {
        let start_pos = self.pos;

        let expr = if let Some(expr) = self.parse_expr() {
            expr
        } else {
            self.pos = start_pos;
            return None;
        };

        let end_span =
            if let Some(semi_token) = self.expect(TokenKind::Semicolon) {
                semi_token.span
            } else if self.at(TokenKind::PhpClose) || self.at(TokenKind::Eof) {
                let span = expr.span();
                Span::at(span.end)
            } else {
                self.error(Diagnostic::error_from_code(
                    ParseDiagnosticCode::expected_token(TokenKind::Semicolon),
                    Span::at(expr.span().end),
                ));
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
