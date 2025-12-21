use crate::ast::{Expr, Stmt};
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_print_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwPrint));

        let kw = self.bump();
        let start = kw.span.start;

        let expr = match self.parse_expr() {
            Some(e) => e,
            None => {
                self.error_and_recover(
                    "expected expression after 'print'",
                    &[
                        TokenKind::Semicolon,
                        TokenKind::PhpClose,
                        TokenKind::RBrace,
                    ],
                );
                Expr::Error { span: kw.span }
            },
        };

        let semi =
            self.expect(TokenKind::Semicolon, "expected ';' after print");
        let end = semi
            .map(|t| t.span.end)
            .or_else(|| self.prev_span().map(|s| s.end))
            .unwrap_or(expr.span().end);

        Some(Stmt::Print {
            expr,
            span: Span { start, end },
        })
    }
}
