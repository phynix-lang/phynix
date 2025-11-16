use crate::ast::{Expr, MatchArm, MatchPattern};
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_match_expr(&mut self) -> Option<Expr> {
        debug_assert!(self.at(TokenKind::KwMatch));

        let match_token = self.bump();
        let start = match_token.span.start;
        let mut last_end = match_token.span.end;

        let lp = self.expect(TokenKind::LParen, "expected '(' after 'match'");
        if let Some(token) = lp.as_ref() {
            last_end = token.span.end;
        }

        let scrutinee = match self.parse_expr() {
            Some(expr) => {
                last_end = expr.span().end;
                expr
            },
            None => {
                self.error_here("expected expression after '('");
                Expr::Error {
                    span: match_token.span,
                }
            },
        };

        let rp =
            self.expect(TokenKind::RParen, "expected ')' after match expr");
        if let Some(token) = rp.as_ref() {
            last_end = token.span.end;
        }

        let lb =
            self.expect(TokenKind::LBrace, "expected '{' to start match arms");
        let lb_start = lb.as_ref().map(|t| t.span.start).unwrap_or(last_end);

        let mut arms: Vec<MatchArm> = Vec::new();

        loop {
            if self.at(TokenKind::RBrace) {
                break;
            }

            let mut patterns: Vec<MatchPattern> = Vec::new();

            if self.at(TokenKind::KwDefault) {
                let d = self.bump();
                patterns.push(MatchPattern::Default { span: d.span });
                last_end = d.span.end;
            } else {
                let first_pat = match self.parse_expr() {
                    Some(expr) => {
                        last_end = expr.span().end;
                        MatchPattern::Expr(expr)
                    },
                    None => {
                        self.error_and_recover(
                            "expected pattern expression or 'default'",
                            &[
                                TokenKind::FatArrow,
                                TokenKind::Comma,
                                TokenKind::RBrace,
                            ],
                        );
                        MatchPattern::Expr(Expr::Error {
                            span: self.prev_span().unwrap_or(Span {
                                start,
                                end: last_end,
                            }),
                        })
                    },
                };
                patterns.push(first_pat);

                while self.eat(TokenKind::Comma) {
                    if let Some(expr) = self.parse_expr() {
                        last_end = expr.span().end;
                        patterns.push(MatchPattern::Expr(expr));
                    } else {
                        self.error_here("expected expression after ',' in match pattern list");
                        break;
                    }
                }
            }

            let arrow = self.expect(
                TokenKind::FatArrow,
                "expected '=>' after match pattern list",
            );
            if let Some(token) = arrow.as_ref() {
                last_end = token.span.end;
            }

            let arm_expr = match self.parse_expr() {
                Some(expr) => {
                    last_end = expr.span().end;
                    expr
                },
                None => {
                    self.error_here("expected expression after '=>'");
                    Expr::Error {
                        span: self.prev_span().unwrap_or(Span {
                            start,
                            end: last_end,
                        }),
                    }
                },
            };

            let mut arm_end = arm_expr.span().end;
            if self.eat(TokenKind::Comma) {
                arm_end = self.prev_span().unwrap().end;
            }

            arms.push(MatchArm {
                patterns,
                expr: arm_expr,
                span: Span {
                    start,
                    end: arm_end,
                },
            });

            if self.at(TokenKind::RBrace) {
                break;
            }
        }

        let rb = self.expect(TokenKind::RBrace, "expected '}' to close match");
        let end = rb
            .as_ref()
            .map(|t| t.span.end)
            .unwrap_or_else(|| lb_start.max(last_end));

        Some(Expr::Match {
            scrutinee: Box::new(scrutinee),
            arms,
            span: Span { start, end },
        })
    }
}
