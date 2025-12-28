use crate::ast::Expr;
use crate::parser::Parser;
use phynix_core::token::TokenKind;
use phynix_core::Span;

impl<'src> Parser<'src> {
    pub(super) fn parse_eval_expr(&mut self) -> Option<Expr> {
        debug_assert!(self.at(TokenKind::KwEval));

        let kw = self.bump();
        let start = kw.span.start;

        let mut last_end = kw.span.end;
        if !self.expect_or_err(TokenKind::LParen, &mut last_end) {
            return None;
        }

        let inner = self.parse_expr_or_err(&mut last_end);

        self.expect_or_err(TokenKind::RParen, &mut last_end);

        Some(Expr::Eval {
            expr: Box::new(inner),
            span: Span {
                start,
                end: last_end,
            },
        })
    }
}
