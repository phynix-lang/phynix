use crate::ast::{Ident, Stmt};
use crate::parser::Parser;
use phynix_core::token::TokenKind;
use phynix_core::Span;

impl<'src> Parser<'src> {
    pub(super) fn parse_const_decl_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwConst));

        let const_token = self.bump();
        let start_pos = const_token.span.start;
        let mut last_end = const_token.span.end;

        let name_token = match self.expect_ident_or_err(&mut last_end) {
            Some(token) => token,
            None => {
                let fake_span = Span::at(last_end);

                return Some(Stmt::ConstDecl {
                    name: Ident { span: fake_span },
                    value: None,
                    span: Span {
                        start: start_pos,
                        end: last_end,
                    },
                });
            },
        };

        let const_ident = Ident {
            span: name_token.span,
        };

        if !self.expect_or_err(TokenKind::Eq, &mut last_end) {
            let span = Span {
                start: start_pos,
                end: name_token.span.end,
            };

            return Some(Stmt::ConstDecl {
                name: const_ident,
                value: None,
                span,
            });
        }

        let value_expr = if self.at(TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expr_or_err(&mut last_end))
        };

        self.expect_or_err(TokenKind::Semicolon, &mut last_end);

        let full_span = Span {
            start: start_pos,
            end: last_end,
        };

        Some(Stmt::ConstDecl {
            name: const_ident,
            value: value_expr,
            span: full_span,
        })
    }
}
