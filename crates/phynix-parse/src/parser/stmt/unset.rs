use crate::ast::{Expr, Stmt};
use crate::parser::Parser;
use phynix_core::Span;
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_unset_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwUnset));

        let kw = self.bump();
        let start = kw.span.start;

        let _lp = self.expect(TokenKind::LParen, "expected '(' after 'unset'");

        let mut exprs: Vec<Expr> = Vec::new();

        if self.eat(TokenKind::RParen) {
            let end = self.prev_span().unwrap().end;
            self.error_span(
                Span { start, end },
                "unset() expects at least 1 argument",
            );

            let semi = self
                .expect(TokenKind::Semicolon, "expected ';' after unset(...)");
            let end = semi.map(|t| t.span.end).unwrap_or(end);

            return Some(Stmt::Unset {
                exprs,
                span: Span { start, end },
            });
        }

        loop {
            if let Some(e) = self.parse_expr() {
                exprs.push(e);
            } else {
                self.error_and_recover(
                    "expected expression in unset(...)",
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

        let _rp =
            self.expect(TokenKind::RParen, "expected ')' after unset(...)");

        let semi =
            self.expect(TokenKind::Semicolon, "expected ';' after unset(...)");
        let end = self.end_pos_or(semi, kw.span.end);

        Some(Stmt::Unset {
            exprs,
            span: Span { start, end },
        })
    }
}
