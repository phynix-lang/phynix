use crate::ast::Expr;
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_eval_expr(&mut self) -> Option<Expr> {
        debug_assert!(self.at(TokenKind::KwEval));

        let kw = self.bump();
        let start = kw.span.start;

        let _lp =
            self.expect(TokenKind::LParen, "expected '(' after 'eval'")?;

        let inner = match self.parse_expr() {
            Some(e) => e,
            None => {
                self.error_and_recover(
                    "expected expression inside eval(...)",
                    &[TokenKind::RParen],
                );
                Expr::Error { span: kw.span }
            },
        };

        let rp = self.expect(TokenKind::RParen, "expected ')' after eval(...)");
        let end = self.end_pos_or(rp, inner.span().end);

        Some(Expr::Eval {
            expr: Box::new(inner),
            span: Span { start, end },
        })
    }
}
