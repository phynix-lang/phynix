use crate::ast::Stmt;
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_return_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwReturn));

        let return_token = self.bump();
        let start = return_token.span.start;
        let mut last_end = return_token.span.end;

        let mut reported = false;

        let maybe_expr = if self.at(TokenKind::Semicolon) {
            None
        } else if let Some(expr) = self.parse_expr() {
            last_end = expr.span().end;
            Some(expr)
        } else {
            self.error_here(
                ParseDiagnosticCode::ExpectedExpression,
                "expected expression after 'return'",
            );
            reported = true;
            None
        };

        let end_span = if self.eat(TokenKind::Semicolon) {
            self.prev_span().unwrap()
        } else {
            if !reported {
                self.error_here(
                    ParseDiagnosticCode::ExpectedToken,
                    "expected ';' after return",
                );
            }
            self.recover_to_any(&[
                TokenKind::Semicolon,
                TokenKind::RBrace,
                TokenKind::Dollar,
                TokenKind::Ident,
                TokenKind::AttrOpen,
            ]);
            Span {
                start: last_end,
                end: last_end,
            }
        };

        Some(Stmt::Return {
            expr: maybe_expr,
            span: Span {
                start,
                end: end_span.end,
            },
        })
    }
}
