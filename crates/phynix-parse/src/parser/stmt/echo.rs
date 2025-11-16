use crate::ast::{Expr, Stmt};
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub fn parse_echo_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwEcho));

        let echo_token = self.bump();
        let start_span = echo_token.span;

        let mut exprs = Vec::new();

        if let Some(first) = self.parse_expr() {
            exprs.push(first);

            loop {
                if !self.eat(TokenKind::Comma) {
                    break;
                }

                if let Some(next) = self.parse_expr() {
                    exprs.push(next);
                } else {
                    self.error_here("expected expression after ',' in echo");
                    break;
                }
            }
        } else {
            self.error_here("expected expression after 'echo'");
        }

        self.skip_trivia_and_cache();

        let end_span = if self.eat(TokenKind::Semicolon) {
            self.prev_span().unwrap()
        } else if self.at(TokenKind::PhpClose) {
            exprs.last().map(|e| e.span()).unwrap_or(start_span)
        } else {
            self.error_and_recover(
                "expected ';' after echo",
                &[
                    TokenKind::Semicolon,
                    TokenKind::PhpClose,
                    TokenKind::RBrace,
                    TokenKind::Dollar,
                    TokenKind::Ident,
                    TokenKind::AttrOpen,
                ],
            );
            let _ = self.eat(TokenKind::Semicolon);
            self.prev_span().unwrap_or_else(|| {
                exprs.last().map(|e| e.span()).unwrap_or(start_span)
            })
        };

        let span = Span {
            start: start_span.start,
            end: end_span.end,
        };

        Some(Stmt::Echo { exprs, span })
    }

    pub(crate) fn parse_echo_open_stmt(&mut self) -> Option<Stmt> {
        let open = self.prev_span().unwrap_or(self.current_span());
        let start = open.start;

        let expr = match self.parse_expr() {
            Some(e) => e,
            None => {
                self.error_here("expected expression after '<?='");
                let fake = self.prev_span().unwrap_or(open);
                Expr::Error { span: fake }
            },
        };

        let _ = self.eat(TokenKind::Semicolon);

        let end = if let Some(token) =
            self.expect(TokenKind::PhpClose, "expected '?>' after '<?= expr'")
        {
            token.span.end
        } else {
            expr.span().end
        };

        let span = Span { start, end };
        Some(Stmt::Echo {
            exprs: vec![expr],
            span,
        })
    }
}
