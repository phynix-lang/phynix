use crate::ast::{Expr, Stmt};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

impl<'src> Parser<'src> {
    pub(super) fn parse_unset_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwUnset));

        let kw = self.bump();
        let start = kw.span.start;
        let mut last_end = kw.span.end;

        self.expect_or_err(TokenKind::LParen, &mut last_end);

        let mut exprs: Vec<Expr> = Vec::new();

        if let Some(rp) = self.expect(TokenKind::RParen) {
            last_end = rp.span.end;
            self.error(Diagnostic::error_from_code(
                ParseDiagnosticCode::ExpectedAtLeastOneArgument,
                Span {
                    start,
                    end: last_end,
                },
            ));

            self.expect_or_err(TokenKind::Semicolon, &mut last_end);

            return Some(Stmt::Unset {
                exprs,
                span: Span {
                    start,
                    end: last_end,
                },
            });
        }

        loop {
            let loop_start = self.current_span().start;
            if let Some(e) = self.parse_expr() {
                last_end = e.span().end;
                exprs.push(e);
            } else {
                self.error_and_recover(
                    Diagnostic::error_from_code(
                        ParseDiagnosticCode::ExpectedExpression,
                        Span::at(loop_start),
                    ),
                    &[TokenKind::Comma, TokenKind::RParen],
                );
            }

            if self.eat(TokenKind::Comma) {
                if self.at(TokenKind::RParen) {
                    break;
                }
                continue;
            }

            break;
        }

        self.expect_or_err(TokenKind::RParen, &mut last_end);
        self.expect_or_err(TokenKind::Semicolon, &mut last_end);

        Some(Stmt::Unset {
            exprs,
            span: Span {
                start,
                end: last_end,
            },
        })
    }
}
