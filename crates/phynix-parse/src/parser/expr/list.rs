use crate::ast::{Expr, ListItemExpr};
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(crate) fn parse_list_destructure_expr(&mut self) -> Option<Expr> {
        debug_assert!(self.at(TokenKind::KwList));

        let kw = self.bump();
        let start = kw.span.start;

        let _lp =
            self.expect(TokenKind::LParen, "expected '(' after 'list'")?;

        let mut items: Vec<ListItemExpr> = Vec::new();

        if self.eat(TokenKind::RParen) {
            let end = self.prev_span().unwrap().end;
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

            let first = self.parse_list_item_expr_or_error(kw.span);

            let (key, value) = if self.eat(TokenKind::FatArrow) {
                let key = first;
                let value = self.parse_list_item_expr_or_error(kw.span);
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

            let _ =
                self.expect(TokenKind::RParen, "expected ')' after list(...)");
            let end = self.prev_span().map(|s| s.end).unwrap_or(item_end);

            return Some(Expr::ListDestructure {
                items,
                span: Span { start, end },
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
    fn parse_list_item_expr_or_error(&mut self, fallback: Span) -> Expr {
        if self.at(TokenKind::KwList) {
            return self.parse_list_destructure_expr().unwrap_or(Expr::Error {
                span: self.prev_span().unwrap_or(fallback),
            });
        }

        if self.at(TokenKind::LBracket) {
            let lb = self.bump().span;
            return self.parse_array_literal(true).unwrap_or(Expr::Error {
                span: self.prev_span().unwrap_or(lb),
            });
        }

        self.parse_expr().unwrap_or(Expr::Error {
            span: self.prev_span().unwrap_or(fallback),
        })
    }
}
