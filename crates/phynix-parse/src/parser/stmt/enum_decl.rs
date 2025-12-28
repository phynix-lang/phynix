use crate::ast::{ClassMember, Stmt, TypeRef};
use crate::parser::Parser;
use phynix_core::token::TokenKind;
use phynix_core::Span;

impl<'src> Parser<'src> {
    pub(super) fn parse_enum_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwEnum));

        let enum_token = self.bump();
        let enum_start = enum_token.span.start;
        let mut last_end = enum_token.span.end;

        let enum_ident = self.expect_ident_ast_or_err(&mut last_end);
        if enum_ident.span.start == enum_ident.span.end {
            return Some(Stmt::Enum {
                name: enum_ident,
                backed_type: None,
                implements: Vec::new(),
                body: Vec::<ClassMember>::new(),
                span: Span {
                    start: enum_start,
                    end: last_end,
                },
            });
        }

        let mut backed_type: Option<TypeRef> = None;
        if self.eat(TokenKind::Colon) {
            if let Some(qn) = self.parse_qualified_name() {
                let ty_span = qn.span;
                last_end = ty_span.end;

                backed_type = Some(TypeRef::Named {
                    name: qn,
                    span: ty_span,
                });
            }
        }

        let (implements_list, impl_last_end) = self.parse_implements_clause();
        if let Some(end) = impl_last_end {
            last_end = end;
        }

        if !self.expect_or_err(TokenKind::LBrace, &mut last_end) {
            let span = Span {
                start: enum_start,
                end: last_end,
            };

            return Some(Stmt::Enum {
                name: enum_ident,
                backed_type,
                implements: implements_list,
                body: Vec::<ClassMember>::new(),
                span,
            });
        }

        let mut body_end_pos = last_end;

        let mut depth = 0;
        loop {
            if self.at(TokenKind::LBrace) {
                let token = self.bump();
                depth += 1;
                body_end_pos = token.span.end;
            } else if self.at(TokenKind::RBrace) {
                let token = self.bump();
                body_end_pos = token.span.end;
                if depth == 0 {
                    break;
                } else {
                    depth -= 1;
                }
            } else if self.peek().is_some() {
                let token = self.bump();
                body_end_pos = token.span.end;
            } else {
                break;
            }
        }

        if body_end_pos < last_end {
            body_end_pos = last_end;
        }

        let enum_span = Span {
            start: enum_start,
            end: body_end_pos,
        };

        Some(Stmt::Enum {
            name: enum_ident,
            backed_type,
            implements: implements_list,
            body: Vec::<ClassMember>::new(), // TODO: real enum members
            span: enum_span,
        })
    }
}
