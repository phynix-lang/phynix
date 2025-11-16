use crate::ast::{Expr, Ident};
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_variable_expr(&mut self) -> Option<Expr> {
        debug_assert!(self.at_variable_start());

        if self.at(TokenKind::VarIdent) {
            let var_token = self.bump();
            let name_span = Span {
                start: var_token.span.start + 1,
                end: var_token.span.end,
            };

            return Some(Expr::VarRef {
                name: Ident { span: name_span },
                span: var_token.span,
            });
        }

        // Fallback: leading '$' (supports $$...$name)
        let first_dollar = self.bump();
        let start_pos = first_dollar.span.start;

        // Support $$$foo etc.
        let mut extra = 0;
        while self.eat(TokenKind::Dollar) {
            extra += 1;
        }

        let (name_span, end_span, levels) = if self.at(TokenKind::VarIdent) {
            let v = self.bump();
            (
                Span {
                    start: v.span.start + 1,
                    end: v.span.end,
                },
                v.span,
                extra + 1,
            )
        } else if let Some(id) =
            self.expect_ident("expected identifier after '$'")
        {
            (id.span, id.span, extra)
        } else {
            self.error_and_recover(
                "expected variable name after '$'",
                &[
                    TokenKind::Semicolon,
                    TokenKind::Comma,
                    TokenKind::RParen,
                    TokenKind::RBracket,
                    TokenKind::RBrace,
                ],
            );
            let fake = self.prev_span().unwrap_or(first_dollar.span);
            let err = Expr::Error { span: fake };
            return Some(self.wrap_variable_levels(err, extra, start_pos));
        };

        let base = Expr::VarRef {
            name: Ident { span: name_span },
            span: Span {
                start: start_pos,
                end: end_span.end,
            },
        };

        Some(self.wrap_variable_levels(base, levels, start_pos))
    }

    fn wrap_variable_levels(
        &self,
        mut expr: Expr,
        extra: u32,
        start_pos: u32,
    ) -> Expr {
        for _ in 0..extra {
            let span = Span {
                start: start_pos,
                end: expr.span().end,
            };
            expr = Expr::VariableVariable {
                target: Box::new(expr),
                span,
            };
        }
        expr
    }

    pub(crate) fn at_variable_start(&self) -> bool {
        self.at_any(&[TokenKind::VarIdent, TokenKind::Dollar])
    }
}
