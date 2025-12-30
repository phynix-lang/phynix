use crate::ast::Stmt;
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

impl<'src> Parser<'src> {
    pub fn parse_echo_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwEcho));

        let echo_token = self.bump();
        let start_span = echo_token.span;
        let mut last_end = echo_token.span.end;

        let mut exprs = Vec::new();

        if let Some(first) = self.parse_expr() {
            last_end = first.span().end;
            exprs.push(first);

            loop {
                let comma_span = self.current_span();
                if !self.eat(TokenKind::Comma) {
                    break;
                }
                last_end = comma_span.end;

                if let Some(next) = self.parse_expr() {
                    last_end = next.span().end;
                    exprs.push(next);
                } else {
                    self.error(Diagnostic::error_from_code(
                        ParseDiagnosticCode::ExpectedExpression,
                        Span::at(last_end),
                    ));
                    break;
                }
            }
        } else {
            self.error(Diagnostic::error_from_code(
                ParseDiagnosticCode::ExpectedExpression,
                Span::at(last_end),
            ));
        }

        self.skip_trivia_and_cache();

        let semi_span = self.current_span();
        let end_span = if self.eat(TokenKind::Semicolon) {
            semi_span
        } else if self.at(TokenKind::PhpClose) || self.at(TokenKind::Eof) {
            exprs.last().map(|e| e.span()).unwrap_or(start_span)
        } else {
            self.error_and_recover(
                Diagnostic::error_from_code(
                    ParseDiagnosticCode::expected_token(TokenKind::Semicolon),
                    Span::at(last_end),
                ),
                &[
                    TokenKind::Semicolon,
                    TokenKind::PhpClose,
                    TokenKind::RBrace,
                    TokenKind::Dollar,
                    TokenKind::Ident,
                    TokenKind::AttrOpen,
                ],
            );
            let recovered_semi_span = self.current_span();
            let _ = self.eat(TokenKind::Semicolon);
            if self.at(TokenKind::Semicolon) {
                recovered_semi_span
            } else {
                exprs.last().map(|e| e.span()).unwrap_or(start_span)
            }
        };

        let span = Span {
            start: start_span.start,
            end: end_span.end,
        };

        Some(Stmt::Echo { exprs, span })
    }

    pub(crate) fn parse_echo_open_stmt(
        &mut self,
        open_span: Span,
    ) -> Option<Stmt> {
        let start = open_span.start;
        let mut last_end = open_span.end;

        let expr = self.parse_expr_or_err(&mut last_end);

        let _ = self.eat(TokenKind::Semicolon);

        self.expect_or_err(TokenKind::PhpClose, &mut last_end);

        let span = Span {
            start,
            end: last_end,
        };
        Some(Stmt::Echo {
            exprs: vec![expr],
            span,
        })
    }
}
