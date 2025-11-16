use crate::ast::Expr;
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_paren_expr(&mut self) -> Option<Expr> {
        let lparen_span = self.prev_span().unwrap();

        let inner_expr = match self.parse_expr() {
            Some(expr) => expr,
            None => {
                self.error_and_recover(
                    "expected expression after '('",
                    &[TokenKind::RParen],
                );

                let inner_span = self.prev_span().unwrap_or(lparen_span);
                let placeholder = Expr::Error { span: inner_span };

                let rparen_span = if let Some(rp_tok) =
                    self.expect(TokenKind::RParen, "expected ')'")
                {
                    rp_tok.span
                } else {
                    self.recover_to_any(&[
                        TokenKind::Semicolon,
                        TokenKind::Comma,
                        TokenKind::RBrace,
                        TokenKind::RParen,
                    ]);

                    inner_span
                };

                let full_span = Span {
                    start: lparen_span.start,
                    end: rparen_span.end,
                };

                return Some(Expr::Parenthesized {
                    inner: Box::new(placeholder),
                    span: full_span,
                });
            },
        };

        let rparen_span = if let Some(rp_tok) =
            self.expect(TokenKind::RParen, "expected ')'")
        {
            rp_tok.span
        } else {
            self.recover_to_any(&[
                TokenKind::Semicolon,
                TokenKind::Comma,
                TokenKind::RBrace,
                TokenKind::RParen,
            ]);

            inner_expr.span()
        };

        let span = Span {
            start: lparen_span.start,
            end: rparen_span.end,
        };

        Some(Expr::Parenthesized {
            inner: Box::new(inner_expr),
            span,
        })
    }
}
