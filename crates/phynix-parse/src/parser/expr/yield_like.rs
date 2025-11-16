use crate::ast::Expr;
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_yield_like_expr(&mut self) -> Option<Expr> {
        debug_assert!(self.at(TokenKind::KwYield));

        let yield_token = self.bump();
        let start = yield_token.span.start;
        let mut last_end = yield_token.span.end;

        if self.at(TokenKind::KwFrom) {
            let f = self.bump();
            last_end = f.span.end;

            let value = if let Some(expr) = self.parse_expr() {
                last_end = expr.span().end;
                expr
            } else {
                self.error_here("expected expression after 'yield from'");
                Expr::Error {
                    span: Span {
                        start: last_end,
                        end: last_end,
                    },
                }
            };

            return Some(Expr::YieldFrom {
                expr: Box::new(value),
                span: Span {
                    start,
                    end: last_end,
                },
            });
        }

        let first = if let Some(expr) = self.parse_expr() {
            last_end = expr.span().end;
            expr
        } else {
            self.error_here("expected expression after 'yield'");
            return Some(Expr::Yield {
                key: None,
                value: Box::new(Expr::Error {
                    span: Span {
                        start: last_end,
                        end: last_end,
                    },
                }),
                span: Span {
                    start,
                    end: last_end,
                },
            });
        };

        if self.eat(TokenKind::FatArrow) {
            let _arrow = self.prev_span().unwrap();
            let value = if let Some(expr) = self.parse_expr() {
                last_end = expr.span().end;
                expr
            } else {
                self.error_and_recover(
                    "expected value after '=>' in yield",
                    &[
                        TokenKind::Semicolon,
                        TokenKind::RParen,
                        TokenKind::Comma,
                        TokenKind::RBrace,
                    ],
                );
                Expr::Error {
                    span: self.prev_span().unwrap_or(Span {
                        start: last_end,
                        end: last_end,
                    }),
                }
            };

            return Some(Expr::Yield {
                key: Some(Box::new(first)),
                value: Box::new(value),
                span: Span {
                    start,
                    end: last_end,
                },
            });
        }

        Some(Expr::Yield {
            key: None,
            value: Box::new(first),
            span: Span {
                start,
                end: last_end,
            },
        })
    }
}
