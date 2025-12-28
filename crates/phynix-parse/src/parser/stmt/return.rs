use crate::ast::Stmt;
use crate::parser::Parser;
use phynix_core::token::TokenKind;
use phynix_core::Span;

impl<'src> Parser<'src> {
    pub(super) fn parse_return_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwReturn));

        let return_token = self.bump();
        let start = return_token.span.start;
        let mut last_end = return_token.span.end;

        let maybe_expr = if self.at(TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expr_or_err(&mut last_end))
        };

        if !self.expect_or_err(TokenKind::Semicolon, &mut last_end) {
            self.recover_to_any(&[
                TokenKind::Semicolon,
                TokenKind::RBrace,
                TokenKind::Dollar,
                TokenKind::Ident,
                TokenKind::AttrOpen,
            ]);
        }

        Some(Stmt::Return {
            expr: maybe_expr,
            span: Span {
                start,
                end: last_end,
            },
        })
    }
}
