use crate::ast::{ClassMember, Stmt};
use crate::parser::Parser;
use phynix_core::token::TokenKind;
use phynix_core::Span;

impl<'src> Parser<'src> {
    pub(super) fn parse_trait_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwTrait));

        let trait_token = self.bump();
        let trait_start = trait_token.span.start;
        let mut last_end = trait_token.span.end;

        let trait_ident = self.expect_ident_ast_or_err(&mut last_end);

        if !self.expect_or_err(TokenKind::LBrace, &mut last_end) {
            let span = Span {
                start: trait_start,
                end: last_end,
            };
            return Some(Stmt::Trait {
                name: trait_ident,
                body: Vec::<ClassMember>::new(),
                span,
            });
        }

        let body_end = self.consume_brace_body(last_end);

        let trait_span = Span {
            start: trait_start,
            end: body_end,
        };

        Some(Stmt::Trait {
            name: trait_ident,
            body: Vec::<ClassMember>::new(), // TODO: members
            span: trait_span,
        })
    }
}
