use crate::ast::{ClassFlags, QualifiedName};
use crate::parser::Parser;
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn consume_brace_body(&mut self, first_lbrace_end: u32) -> u32 {
        let mut depth = 1;
        let mut body_end = first_lbrace_end;

        while !self.eof() && depth > 0 {
            if self.at(TokenKind::LBrace) {
                let t = self.bump();
                depth += 1;
                body_end = t.span.end;
                continue;
            }

            if self.at(TokenKind::RBrace) {
                let t = self.bump();
                depth -= 1;
                body_end = t.span.end;
                continue;
            }

            self.bump();
        }

        body_end
    }

    pub(super) fn parse_class_flags(&mut self) -> ClassFlags {
        let mut flags = ClassFlags::empty();

        loop {
            if self.at(TokenKind::KwAbstract) {
                let _abstract_token = self.bump();
                flags |= ClassFlags::ABSTRACT;
                continue;
            }

            if self.at(TokenKind::KwFinal) {
                let _final_token = self.bump();
                flags |= ClassFlags::FINAL;
                continue;
            }

            if self.at(TokenKind::KwReadonly) {
                let _readonly_token = self.bump();
                flags |= ClassFlags::READONLY;
                continue;
            }

            break;
        }

        flags
    }

    pub(super) fn parse_implements_clause(
        &mut self,
    ) -> (Vec<QualifiedName>, Option<u32>) {
        let mut interfaces = Vec::new();
        let mut last_end = None;

        if self.at(TokenKind::KwImplements) {
            let _implements_token = self.bump();

            loop {
                if let Some(iface_qn) = self.parse_qualified_name(
                    "expected interface name after 'implements'",
                ) {
                    last_end = Some(iface_qn.span.end);
                    interfaces.push(iface_qn);
                } else {
                    break;
                }

                if self.eat(TokenKind::Comma) {
                    continue;
                }
                break;
            }
        }

        (interfaces, last_end)
    }
}
