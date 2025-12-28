use crate::ast::{Ident, Stmt};
use crate::parser::Parser;
use phynix_core::token::TokenKind;
use phynix_core::Span;

impl<'src> Parser<'src> {
    pub fn parse_label_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(
            self.at(TokenKind::Ident) && self.at_nth(1, TokenKind::Colon)
        );

        let name_token = self.bump();
        let mut last_end = name_token.span.end;

        self.expect_or_err(TokenKind::Colon, &mut last_end);

        let span = Span {
            start: name_token.span.start,
            end: last_end,
        };

        Some(Stmt::Label {
            name: Ident {
                span: name_token.span,
            },
            span,
        })
    }
}
