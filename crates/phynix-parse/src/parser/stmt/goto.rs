use crate::ast::{Ident, Stmt};
use crate::parser::Parser;
use phynix_core::token::TokenKind;
use phynix_core::Span;

impl<'src> Parser<'src> {
    pub fn parse_goto_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwGoto));

        let kw = self.bump();
        let mut last_end = kw.span.end;
        let ident = self.expect_ident_or_err(&mut last_end)?;
        last_end = ident.span.end;

        self.expect_or_err(TokenKind::Semicolon, &mut last_end);

        let span = Span {
            start: kw.span.start,
            end: last_end,
        };
        Some(Stmt::Goto {
            target: Ident { span: ident.span },
            span,
        })
    }
}
