use crate::ast::{ClassMember, QualifiedName, Stmt};
use crate::parser::Parser;
use phynix_core::token::TokenKind;
use phynix_core::Span;

impl<'src> Parser<'src> {
    pub(super) fn parse_interface_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwInterface));

        let interface_token = self.bump();
        let iface_start = interface_token.span.start;
        let mut last_end = interface_token.span.end;

        let iface_ident = self.expect_ident_ast_or_err(&mut last_end);

        let mut extends_list: Vec<QualifiedName> = Vec::new();
        if self.at(TokenKind::KwExtends) {
            let _extends_token = self.bump();

            loop {
                if let Some(parent_qn) = self.parse_qualified_name() {
                    last_end = parent_qn.span.end;
                    extends_list.push(parent_qn);
                } else {
                    if self.eat(TokenKind::Comma) {
                        continue;
                    }
                    break;
                }

                if self.eat(TokenKind::Comma) {
                    continue;
                }
                break;
            }
        }

        if !self.expect_or_err(TokenKind::LBrace, &mut last_end) {
            let span = Span {
                start: iface_start,
                end: last_end,
            };
            return Some(Stmt::Interface {
                name: iface_ident,
                extends: extends_list,
                body: Vec::<ClassMember>::new(),
                span,
            });
        }

        let body_end = self.consume_brace_body(last_end);

        let iface_span = Span {
            start: iface_start,
            end: body_end,
        };

        Some(Stmt::Interface {
            name: iface_ident,
            extends: extends_list,
            body: Vec::<ClassMember>::new(), // TODO: members
            span: iface_span,
        })
    }
}
