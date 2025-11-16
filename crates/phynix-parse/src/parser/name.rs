use crate::ast::{Ident, QualifiedName};
use crate::parser::Parser;
use phynix_core::Span;
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_qualified_name(
        &mut self,
        err_msg: &'static str,
    ) -> Option<QualifiedName> {
        let mut absolute = false;
        let start_pos = if self.at(TokenKind::Backslash) {
            let backslash = self.bump();
            absolute = true;
            backslash.span.start
        } else {
            self.current_span().start
        };

        let first_ident_token = self.expect_ident(err_msg)?;

        let mut parts = Vec::new();
        parts.push(Ident {
            span: first_ident_token.span,
        });
        let mut end_pos = first_ident_token.span.end;

        loop {
            if !self.at(TokenKind::Backslash) {
                break;
            }

            self.bump();
            if self.at(TokenKind::LBrace) {
                break;
            }

            if let Some(next_ident_token) =
                self.expect_ident("expected identifier after '\\'")
            {
                parts.push(Ident {
                    span: next_ident_token.span,
                });
                end_pos = next_ident_token.span.end;
                continue;
            } else {
                break;
            }
        }

        Some(QualifiedName {
            absolute,
            parts,
            span: Span {
                start: start_pos,
                end: end_pos,
            },
        })
    }
}
