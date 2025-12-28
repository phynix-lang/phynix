use crate::ast::{Stmt, SwitchCase};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

impl<'src> Parser<'src> {
    pub(super) fn parse_switch_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwSwitch));

        let kw = self.bump();
        let start = kw.span.start;
        let mut last_end = kw.span.end;

        self.expect_or_err(TokenKind::LParen, &mut last_end);

        let cond = self.parse_expr_or_err(&mut last_end);

        self.expect_or_err(TokenKind::RParen, &mut last_end);

        let mut cases: Vec<SwitchCase> = Vec::new();

        // Two syntaxes:
        // switch (...) { ... }
        // switch (...): ... endswitch;
        if let Some(lb) = self.expect(TokenKind::LBrace) {
            last_end = lb.span.end;
            self.parse_switch_cases(
                TokenKind::RBrace,
                &mut last_end,
                &mut cases,
            );

            self.expect_or_err(TokenKind::RBrace, &mut last_end);
        } else if let Some(colon) = self.expect(TokenKind::Colon) {
            last_end = colon.span.end;
            self.parse_switch_cases(
                TokenKind::KwEndSwitch,
                &mut last_end,
                &mut cases,
            );

            self.expect_or_err(TokenKind::KwEndSwitch, &mut last_end);
            let _ = self.eat(TokenKind::Semicolon);
        } else {
            self.error(Diagnostic::error_from_code(
                ParseDiagnosticCode::expected_tokens([
                    TokenKind::LBrace,
                    TokenKind::Colon,
                ]),
                Span::at(last_end),
            ));
        }

        let span = Span {
            start,
            end: last_end,
        };

        Some(Stmt::Switch { cond, cases, span })
    }

    fn parse_switch_cases(
        &mut self,
        terminator: TokenKind,
        last_end: &mut u32,
        cases: &mut Vec<SwitchCase>,
    ) {
        loop {
            if self.eof() || self.at(terminator) {
                break;
            }

            if self.at(TokenKind::KwCase) || self.at(TokenKind::KwDefault) {
                let is_default = self.at(TokenKind::KwDefault);
                let kw = self.bump();
                let case_start = kw.span.start;
                *last_end = kw.span.end;

                let condition = if is_default {
                    None
                } else {
                    Some(self.parse_expr_or_err(last_end))
                };

                if !self.eat(TokenKind::Colon)
                    && !self.eat(TokenKind::Semicolon)
                {
                    self.error(Diagnostic::error_from_code(
                        ParseDiagnosticCode::expected_tokens([
                            TokenKind::Colon,
                            TokenKind::Semicolon,
                        ]),
                        Span::at(*last_end),
                    ));
                }

                let (body, end) = self.parse_until_any(&[
                    TokenKind::KwCase,
                    TokenKind::KwDefault,
                    terminator,
                ]);
                *last_end = end;

                let span = Span {
                    start: case_start,
                    end,
                };

                cases.push(SwitchCase {
                    condition,
                    body,
                    span,
                });
                continue;
            }

            let before = self.pos;
            if let Some(stmt) = self.parse_stmt() {
                *last_end = stmt.span().end;

                if let Some(last) = cases.last_mut() {
                    last.body.items.push(stmt);
                    last.body.span.end = *last_end;
                } else {
                    self.error(Diagnostic::error_from_code(
                        ParseDiagnosticCode::expected_tokens([
                            TokenKind::KwCase,
                            TokenKind::KwDefault,
                        ]),
                        Span::at(*last_end),
                    ));
                }
            } else {
                if self.pos == before {
                    if self.eof() {
                        break;
                    }
                    let _ = self.bump();
                }
            }
        }
    }
}
