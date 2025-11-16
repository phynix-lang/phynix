use crate::ast::{ClassMember, Ident, QualifiedName, Stmt};
use crate::parser::Parser;
use phynix_core::Span;
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_interface_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwInterface));

        let interface_token = self.bump();
        let iface_start = interface_token.span.start;
        let mut last_end = interface_token.span.end;

        let iface_ident = if let Some(name_token) =
            self.expect_ident("expected interface name after 'interface'")
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

        let mut extends_list: Vec<QualifiedName> = Vec::new();
        if self.at(TokenKind::KwExtends) {
            let _extends_token = self.bump();

            loop {
                if let Some(parent_qn) = self.parse_qualified_name(
                    "expected interface name after 'extends'",
                ) {
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

        let lbrace_token = self
            .expect(TokenKind::LBrace, "expected '{' to start interface body");

        if lbrace_token.is_none() {
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

        let lbrace_token = lbrace_token.unwrap();
        let body_end = self.consume_brace_body(lbrace_token.span.end);

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
