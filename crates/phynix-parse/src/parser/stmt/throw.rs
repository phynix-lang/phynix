use crate::ast::Stmt;
use crate::parser::Parser;
use phynix_core::token::TokenKind;
use phynix_core::Span;

impl<'src> Parser<'src> {
    pub(super) fn parse_throw_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwThrow));

        let throw_token = self.bump();
        let start_pos = throw_token.span.start;
        let mut last_end = throw_token.span.end;

        let thrown_expr = self.parse_expr_or_err(&mut last_end);

        if !self.at(TokenKind::PhpClose)
            && !self.expect_or_err(TokenKind::Semicolon, &mut last_end)
        {
            self.recover_to_any(&[
                TokenKind::Semicolon,
                TokenKind::PhpClose,
                TokenKind::RBrace,
                TokenKind::Dollar,
                TokenKind::Ident,
                TokenKind::AttrOpen,
            ]);
            let _ = self.eat(TokenKind::Semicolon);
        }

        Some(Stmt::Throw {
            expr: thrown_expr,
            span: Span {
                start: start_pos,
                end: last_end,
            },
        })
    }
}
