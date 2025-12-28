use crate::ast::{Expr, ListItemExpr};
use crate::parser::Parser;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

impl<'src> Parser<'src> {
    pub(crate) fn parse_list_destructure_expr(&mut self) -> Option<Expr> {
        debug_assert!(self.at(TokenKind::KwList));

        let kw = self.bump();
        let start = kw.span.start;

        let mut last_end = kw.span.end;
        if !self.expect_or_err(TokenKind::LParen, &mut last_end) {
            return Some(Expr::Error {
                span: Span::at(last_end),
            });
        }

        let mut items: Vec<ListItemExpr> = Vec::new();

        if self.eat(TokenKind::RParen) {
            let end = self.current_span().start;
            return Some(Expr::ListDestructure {
                items,
                span: Span { start, end },
            });
        }

        loop {
            if self.at(TokenKind::RParen) {
                let end = self.bump().span.end;
                return Some(self.finish_list_destructure(start, end, items));
            }

            // Empty slot: list(, $b)
            if self.at(TokenKind::Comma) {
                let comma = self.bump();
                let sp = comma.span;
                items.push(ListItemExpr {
                    key: None,
                    value: None,
                    span: sp,
                });
                continue;
            }

            let item_start = self.current_span().start;

            let first = self.parse_list_item_expr_or_error();

            let (key, value) = if self.eat(TokenKind::FatArrow) {
                let key = first;
                let value = self.parse_list_item_expr_or_error();
                (Some(key), Some(value))
            } else {
                (None, Some(first))
            };

            let item_end = value
                .as_ref()
                .map(|e| e.span().end)
                .or_else(|| key.as_ref().map(|e| e.span().end))
                .unwrap_or(item_start);

            items.push(ListItemExpr {
                key,
                value,
                span: Span {
                    start: item_start,
                    end: item_end,
                },
            });

            if self.eat(TokenKind::Comma) {
                if self.at(TokenKind::RParen) {
                    let end = self.bump().span.end;
                    return Some(
                        self.finish_list_destructure(start, end, items),
                    );
                }
                continue;
            }

            if self.eat(TokenKind::RParen) {
                let end = self.current_span().start;
                return Some(Expr::ListDestructure {
                    items,
                    span: Span { start, end },
                });
            }

            let mut last_end_rp = item_end;
            self.expect_or_err(TokenKind::RParen, &mut last_end_rp);
            return Some(Expr::ListDestructure {
                items,
                span: Span {
                    start,
                    end: last_end_rp,
                },
            });
        }
    }

    #[inline]
    fn finish_list_destructure(
        &self,
        start: u32,
        end: u32,
        items: Vec<ListItemExpr>,
    ) -> Expr {
        Expr::ListDestructure {
            items,
            span: Span { start, end },
        }
    }

    #[inline]
    fn parse_list_item_expr_or_error(&mut self) -> Expr {
        let start = self.current_span().start;

        if self.at(TokenKind::KwList) {
            return self.parse_list_destructure_expr().unwrap_or_else(|| {
                let sp = Span { start, end: start };
                Expr::Error { span: sp }
            });
        }

        if self.at(TokenKind::LBracket) {
            let lb_span = self.current_span();
            self.bump();
            return self
                .parse_array_literal(true, Some(lb_span))
                .unwrap_or_else(|| {
                    let sp = Span { start, end: start };
                    Expr::Error { span: sp }
                });
        }

        self.parse_expr().unwrap_or_else(|| {
            let sp = Span { start, end: start };
            Expr::Error { span: sp }
        })
    }
}
