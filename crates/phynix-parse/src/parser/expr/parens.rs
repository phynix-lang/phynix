use crate::ast::Expr;
use crate::parser::Parser;
use phynix_core::token::TokenKind;
use phynix_core::Span;

impl<'src> Parser<'src> {
    pub(super) fn parse_paren_expr(
        &mut self,
        lparen_span: Span,
    ) -> Option<Expr> {
        let mut last_end = lparen_span.end;
        let inner_expr = self.parse_expr_or_err(&mut last_end);

        if !self.expect_or_err(TokenKind::RParen, &mut last_end) {
            self.recover_to_any(&[
                TokenKind::Semicolon,
                TokenKind::Comma,
                TokenKind::RBrace,
                TokenKind::RParen,
            ]);
        }

        Some(Expr::Parenthesized {
            inner: Box::new(inner_expr),
            span: Span {
                start: lparen_span.start,
                end: last_end,
            },
        })
    }
}
