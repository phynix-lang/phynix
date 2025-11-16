use crate::ast::{
    ClassFlags, ClassMember, ClassNameRef, Ident, QualifiedName, Stmt,
};
use crate::parser::Parser;
use phynix_core::Span;
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_class_stmt(
        &mut self,
        flags: ClassFlags,
    ) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwClass));

        let class_token = self.bump();
        let class_start = class_token.span.start;

        let (class_name_qn, mut last_end) = if let Some(name_tok) =
            self.expect_ident("expected class name after 'class'")
        {
            let ident_span = name_tok.span;

            let qn = QualifiedName {
                absolute: false,
                parts: vec![Ident { span: ident_span }],
                span: ident_span,
            };

            (qn, ident_span.end)
        } else {
            let fake_span = Span {
                start: class_token.span.end,
                end: class_token.span.end,
            };

            let qn = QualifiedName {
                absolute: false,
                parts: vec![Ident { span: fake_span }],
                span: fake_span,
            };

            (qn, class_token.span.end)
        };

        let mut extends_name: Option<QualifiedName> = None;
        if self.at(TokenKind::KwExtends) {
            let _extends_token = self.bump();

            if let Some(parent_qn) = self.parse_qualified_name(
                "expected parent class name after 'extends'",
            ) {
                last_end = parent_qn.span.end;
                extends_name = Some(parent_qn);
            }
        }

        let (implements_list, impl_last_end) = self.parse_implements_clause();
        if let Some(end) = impl_last_end {
            last_end = end;
        }

        let lbrace_token =
            self.expect(TokenKind::LBrace, "expected '{' to start class body");

        if lbrace_token.is_none() {
            let span = Span {
                start: class_start,
                end: last_end,
            };

            return Some(Stmt::Class {
                flags,
                name: ClassNameRef::Qualified(class_name_qn),
                extends: extends_name,
                implements: implements_list,
                body: Vec::<ClassMember>::new(),
                span,
            });
        }

        let lbrace_token = lbrace_token.unwrap();
        let body_start = lbrace_token.span.end;

        let (body_end_pos, _members) = self.consume_class_body(body_start);

        let class_span = Span {
            start: class_start,
            end: body_end_pos,
        };

        Some(Stmt::Class {
            flags,
            name: ClassNameRef::Qualified(class_name_qn),
            extends: extends_name,
            implements: implements_list,
            body: Vec::<ClassMember>::new(),
            span: class_span,
        })
    }

    fn consume_class_body(
        &mut self,
        body_start_pos: u32,
    ) -> (u32, Vec<ClassMember>) {
        let members: Vec<ClassMember> = Vec::new();
        let mut depth: i8 = 0;

        while let Some(token) = self.peek() {
            match token.kind {
                TokenKind::RBrace if depth == 0 => {
                    let close = self.bump();
                    return (close.span.end, members);
                },
                TokenKind::LBrace => {
                    depth += 1;
                    self.bump();
                },
                TokenKind::RBrace => {
                    depth = depth.saturating_sub(1);
                    self.bump();
                },
                TokenKind::Semicolon if depth == 0 => {
                    self.bump();
                },
                _ => {
                    self.bump();
                },
            }
        }

        (body_start_pos, members)
    }
}
