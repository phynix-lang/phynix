use crate::ast::{Expr, Stmt, SwitchCase};
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_switch_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwSwitch));

        let kw = self.bump();
        let start = kw.span.start;
        let mut last_end = kw.span.end;

        if let Some(lp) =
            self.expect(TokenKind::LParen, "expected '(' after 'switch'")
        {
            last_end = lp.span.end;
        }

        let cond = if let Some(expr) = self.parse_expr() {
            last_end = expr.span().end;
            expr
        } else {
            self.error_here("expected expression in switch condition");
            Expr::Error {
                span: Span {
                    start: last_end,
                    end: last_end,
                },
            }
        };

        if let Some(rp) = self
            .expect(TokenKind::RParen, "expected ')' after switch condition")
        {
            last_end = rp.span.end;
        }

        let mut cases: Vec<SwitchCase> = Vec::new();

        // Two syntaxes:
        // switch (...) { ... }
        // switch (...): ... endswitch;
        if self.eat(TokenKind::LBrace) {
            last_end = self.prev_span().unwrap().end;
            self.parse_switch_cases(
                TokenKind::RBrace,
                &mut last_end,
                &mut cases,
            );

            if let Some(rb) =
                self.expect(TokenKind::RBrace, "expected '}' to close switch")
            {
                last_end = rb.span.end;
            }
        } else if self.eat(TokenKind::Colon) {
            last_end = self.prev_span().unwrap().end;
            self.parse_switch_cases(
                TokenKind::KwEndSwitch,
                &mut last_end,
                &mut cases,
            );

            let end_token =
                self.expect(TokenKind::KwEndSwitch, "expected 'endswitch'");
            if let Some(tok) = end_token {
                last_end = tok.span.end;
            }
            let _ = self.eat(TokenKind::Semicolon);
        } else {
            self.error_here("expected '{' or ':' after switch condition");
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

                let condition = if is_default {
                    None
                } else {
                    match self.parse_expr() {
                        Some(expr) => {
                            *last_end = expr.span().end;
                            Some(expr)
                        },
                        None => {
                            self.error_here("expected expression after 'case'");
                            None
                        },
                    }
                };

                if !self.eat(TokenKind::Colon)
                    && !self.eat(TokenKind::Semicolon)
                {
                    self.error_here(
                        "expected ':' or ';' after case/default label",
                    );
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
                    self.error_here("expected 'case' or 'default' in switch");
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
