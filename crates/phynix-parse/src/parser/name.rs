use crate::ast::{Ident, QualifiedName};
use crate::parser::Parser;
use phynix_core::token::TokenKind;
use phynix_core::Span;

impl<'src> Parser<'src> {
    pub(super) fn parse_qualified_name(&mut self) -> Option<QualifiedName> {
        let mut absolute = false;
        let (start_pos, mut last_end) = if self.at(TokenKind::Backslash) {
            let backslash = self.bump();
            absolute = true;
            (backslash.span.start, backslash.span.end)
        } else {
            let pos = self.current_span().start;
            (pos, pos)
        };

        let first_ident_token = self.expect_ident_or_err(&mut last_end)?;

        let mut parts = Vec::new();
        parts.push(Ident {
            span: first_ident_token.span,
        });
        last_end = first_ident_token.span.end;

        loop {
            if !self.at(TokenKind::Backslash) {
                break;
            }

            let bs = self.bump();
            last_end = bs.span.end;
            if self.at(TokenKind::LBrace) {
                break;
            }

            if let Some(next_ident_token) =
                self.expect_ident_or_err(&mut last_end)
            {
                parts.push(Ident {
                    span: next_ident_token.span,
                });
                last_end = next_ident_token.span.end;
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
                end: last_end,
            },
        })
    }
}
