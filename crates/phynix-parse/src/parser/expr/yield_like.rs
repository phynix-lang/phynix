use crate::ast::Expr;
use crate::parser::Parser;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

impl<'src> Parser<'src> {
    pub(super) fn parse_yield_like_expr(&mut self) -> Option<Expr> {
        debug_assert!(self.at(TokenKind::KwYield));

        let yield_token = self.bump();
        let start = yield_token.span.start;
        let mut last_end = yield_token.span.end;

        if self.at(TokenKind::KwFrom) {
            let f = self.bump();
            last_end = f.span.end;

            let value = self.parse_expr_or_err(&mut last_end);

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
            return Some(Expr::Yield {
                key: None,
                value: Box::new(self.parse_expr_or_err(&mut last_end)),
                span: Span {
                    start,
                    end: last_end,
                },
            });
        };

        if self.eat(TokenKind::FatArrow) {
            let mut arrow_end = self.current_span().start;
            let value = self.parse_expr_or_err(&mut arrow_end);
            last_end = arrow_end;

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
