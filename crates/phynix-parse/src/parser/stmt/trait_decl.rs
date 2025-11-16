use crate::ast::{ClassMember, Ident, Stmt};
use crate::parser::Parser;
use phynix_core::Span;
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_trait_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwTrait));

        let trait_token = self.bump();
        let trait_start = trait_token.span.start;
        let mut last_end = trait_token.span.end;

        let trait_ident = if let Some(name_token) =
            self.expect_ident("expected trait name after 'trait'")
        {
            last_end = name_token.span.end;
            Ident {
                span: name_token.span,
            }
        } else {
            let fake = Span {
                start: last_end,
                end: last_end,
            };
            Ident { span: fake }
        };

        let lbrace_token =
            self.expect(TokenKind::LBrace, "expected '{' to start trait body");

        if lbrace_token.is_none() {
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

        let lbrace_token = lbrace_token.unwrap();
        let body_end = self.consume_brace_body(lbrace_token.span.end);

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
