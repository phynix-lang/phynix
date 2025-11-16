use crate::ast::{Ident, Stmt};
use crate::parser::Parser;
use phynix_core::Span;
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub fn parse_goto_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwGoto));

        let kw = self.bump();
        let ident = self.expect_ident("expected label after 'goto'")?;
        let semi =
            self.expect(TokenKind::Semicolon, "expected ';' after goto")?;
        let span = Span {
            start: kw.span.start,
            end: semi.span.end,
        };
        Some(Stmt::Goto {
            target: Ident { span: ident.span },
            span,
        })
    }
}
