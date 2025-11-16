use crate::ast::{Ident, Stmt};
use crate::parser::Parser;
use phynix_core::Span;
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub fn parse_label_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(
            self.at(TokenKind::Ident) && self.at_nth(1, TokenKind::Colon)
        );

        let name_token = self.bump();
        let _ = self.expect(TokenKind::Colon, "expected ':' after label");
        let span = Span {
            start: name_token.span.start,
            end: self.prev_span().unwrap().end,
        };

        Some(Stmt::Label {
            name: Ident {
                span: name_token.span,
            },
            span,
        })
    }
}
