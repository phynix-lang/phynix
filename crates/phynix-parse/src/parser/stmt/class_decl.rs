use crate::ast::{
    AttributeGroup, Block, ClassFlags, ClassMember, ClassNameRef, Expr,
    HookBody, Ident, MemberFlags, PropertyHookGet, PropertyHookSet,
    PropertyHooks, QualifiedName, Stmt, Visibility,
};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::Span;

impl<'src> Parser<'src> {
    pub(super) fn parse_class_stmt(
        &mut self,
        flags: ClassFlags,
        attributes: Vec<AttributeGroup>,
    ) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwClass));

        let class_token = self.bump();
        let class_start = class_token.span.start;

        let mut last_end = class_token.span.end;
        let class_ident = self.expect_ident_ast_or_err(&mut last_end);
        let class_ident_span = class_ident.span;
        let class_name_qn = QualifiedName {
            absolute: false,
            parts: vec![class_ident],
            span: class_ident_span,
        };

        let mut extends_name: Option<QualifiedName> = None;
        if self.at(TokenKind::KwExtends) {
            let _extends_token = self.bump();

            if let Some(parent_qn) = self.parse_qualified_name() {
                last_end = parent_qn.span.end;
                extends_name = Some(parent_qn);
            }
        }

        let (implements_list, impl_last_end) = self.parse_implements_clause();
        if let Some(end) = impl_last_end {
            last_end = end;
        }

        if !self.expect_or_err(TokenKind::LBrace, &mut last_end) {
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
                attributes,
                span,
            });
        }

        let body_start = last_end;

        let (body_end_pos, members) = self.consume_class_body(body_start);

        let class_span = Span {
            start: class_start,
            end: body_end_pos,
        };

        Some(Stmt::Class {
            flags,
            name: ClassNameRef::Qualified(class_name_qn),
            extends: extends_name,
            implements: implements_list,
            body: members,
            attributes,
            span: class_span,
        })
    }

    fn consume_class_body(
        &mut self,
        body_start_pos: u32,
    ) -> (u32, Vec<ClassMember>) {
        let mut members: Vec<ClassMember> = Vec::new();

        while !self.eof() && !self.at(TokenKind::RBrace) {
            if self.eat(TokenKind::Semicolon) {
                continue;
            }

            if self.at(TokenKind::KwUse) {
                if let Some(trait_use) = self.parse_trait_use() {
                    members.push(trait_use);
                }
                continue;
            }

            if let Some(member) = self.parse_class_member() {
                members.push(member);
            } else {
                let err_pos = self.current_span().start;
                self.error_and_recover(
                    Diagnostic::error_from_code(
                        ParseDiagnosticCode::ExpectedStatement,
                        Span::at(err_pos),
                    ),
                    &[
                        TokenKind::KwPublic,
                        TokenKind::KwProtected,
                        TokenKind::KwPrivate,
                        TokenKind::KwStatic,
                        TokenKind::KwFinal,
                        TokenKind::KwAbstract,
                        TokenKind::KwReadonly,
                        TokenKind::KwFunction,
                        TokenKind::KwConst,
                        TokenKind::VarIdent,
                        TokenKind::RBrace,
                    ],
                );
                if self.at(TokenKind::RBrace) {
                    break;
                }
            }
        }

        let mut end = body_start_pos;
        self.expect_or_err(TokenKind::RBrace, &mut end);

        (end, members)
    }

    fn parse_class_member(&mut self) -> Option<ClassMember> {
        let _attrs = if self.at(TokenKind::AttrOpen) {
            self.parse_attribute_group_list().unwrap_or_default()
        } else {
            Vec::new()
        };

        let start_pos = self.current_span().start;

        let mut flags = MemberFlags::empty();
        let mut set_visibility: Option<Visibility> = None;
        let mut saw_visibility = false;

        loop {
            match self.kind() {
                TokenKind::KwPublic => {
                    if self.at_set_visibility_clause() {
                        set_visibility =
                            Some(self.bump_set_visibility_clause());
                    } else {
                        self.bump();
                        flags |= MemberFlags::PUBLIC;
                        saw_visibility = true;
                    }
                },
                TokenKind::KwProtected => {
                    if self.at_set_visibility_clause() {
                        set_visibility =
                            Some(self.bump_set_visibility_clause());
                    } else {
                        self.bump();
                        flags |= MemberFlags::PROTECTED;
                        saw_visibility = true;
                    }
                },
                TokenKind::KwPrivate => {
                    if self.at_set_visibility_clause() {
                        set_visibility =
                            Some(self.bump_set_visibility_clause());
                    } else {
                        self.bump();
                        flags |= MemberFlags::PRIVATE;
                        saw_visibility = true;
                    }
                },
                TokenKind::KwStatic => {
                    self.bump();
                    flags |= MemberFlags::STATIC;
                },
                TokenKind::KwFinal => {
                    self.bump();
                    flags |= MemberFlags::FINAL;
                },
                TokenKind::KwAbstract => {
                    self.bump();
                    flags |= MemberFlags::ABSTRACT;
                },
                TokenKind::KwReadonly => {
                    self.bump();
                    flags |= MemberFlags::READONLY;
                },
                TokenKind::KwVar => {
                    self.bump();
                    // 'var' is equivalent to 'public' for properties.
                    saw_visibility = true;
                    flags |= MemberFlags::PUBLIC;
                },
                _ => break,
            }
        }

        if !saw_visibility {
            flags |= MemberFlags::PUBLIC;
        }

        if self.at(TokenKind::KwConst) {
            return self.parse_class_const(start_pos, flags);
        }

        if self.at(TokenKind::KwFunction) {
            return self.parse_class_method(start_pos, flags);
        }

        self.parse_class_property(start_pos, flags, set_visibility)
    }

    fn parse_class_const(
        &mut self,
        start_pos: u32,
        flags: MemberFlags,
    ) -> Option<ClassMember> {
        debug_assert!(self.at(TokenKind::KwConst));
        let _const_tok = self.bump();

        let mut last_end = _const_tok.span.end;
        let name = self.expect_ident_ast_or_err(&mut last_end);

        if !self.expect_or_err(TokenKind::Eq, &mut last_end) {
            return Some(ClassMember::Const {
                name,
                flags,
                value: Expr::Error {
                    span: Span::at(last_end),
                },
                span: Span {
                    start: start_pos,
                    end: last_end,
                },
            });
        }

        let value = self.parse_expr_or_err(&mut last_end);
        self.expect_or_err(TokenKind::Semicolon, &mut last_end);

        Some(ClassMember::Const {
            name,
            flags,
            value,
            span: Span {
                start: start_pos,
                end: last_end,
            },
        })
    }

    fn parse_class_method(
        &mut self,
        start_pos: u32,
        flags: MemberFlags,
    ) -> Option<ClassMember> {
        debug_assert!(self.at(TokenKind::KwFunction));
        let fn_tok = self.bump();

        let mut last_end = fn_tok.span.end;
        let _by_ref = self.eat(TokenKind::Amp);
        let name = self.expect_ident_ast_or_err(&mut last_end);

        self.expect_or_err(TokenKind::LParen, &mut last_end);

        let mut params = Vec::new();
        if !self.at(TokenKind::RParen) {
            loop {
                if let Some(param) = self.parse_single_param() {
                    params.push(param);
                } else {
                    break;
                }

                if self.eat(TokenKind::Comma) {
                    if self.at(TokenKind::RParen) {
                        break;
                    }
                    continue;
                }
                break;
            }
        }
        self.expect_or_err(TokenKind::RParen, &mut last_end);

        let (return_type, end) = self.parse_optional_return_type(last_end);
        last_end = end;

        let mut body = None;
        if self.at(TokenKind::LBrace) {
            let lb = self.bump();
            if let Some((block, block_end)) =
                self.parse_block_after_lbrace(lb.span.start)
            {
                body = Some(block);
                last_end = block_end;
            }
        } else {
            self.expect_or_err(TokenKind::Semicolon, &mut last_end);
        }

        Some(ClassMember::Method {
            name,
            flags,
            params,
            return_type,
            body,
            span: Span {
                start: start_pos,
                end: last_end,
            },
        })
    }

    fn parse_class_property(
        &mut self,
        start_pos: u32,
        flags: MemberFlags,
        set_visibility: Option<Visibility>,
    ) -> Option<ClassMember> {
        let (type_annotation, mut last_end) = self.parse_param_type_prefix();

        let name_tok = match self.expect(TokenKind::VarIdent) {
            Some(t) => t,
            None => {
                self.error(Diagnostic::error_from_code(
                    ParseDiagnosticCode::expected_token(TokenKind::VarIdent),
                    Span::at(last_end),
                ));
                return Some(ClassMember::Property {
                    name: Ident {
                        span: Span::at(last_end),
                    },
                    flags,
                    set_visibility,
                    type_annotation,
                    default: None,
                    hooks: None,
                    span: Span {
                        start: start_pos,
                        end: last_end,
                    },
                });
            },
        };

        let name = Ident {
            span: name_tok.span,
        };
        last_end = name_tok.span.end;

        let mut default = None;
        if self.eat(TokenKind::Eq) {
            default = Some(self.parse_expr_or_err(&mut last_end));
        }

        let hooks = if self.at(TokenKind::LBrace) {
            Some(self.parse_property_hooks(&mut last_end))
        } else {
            self.expect_or_err(TokenKind::Semicolon, &mut last_end);
            None
        };

        Some(ClassMember::Property {
            name,
            flags,
            set_visibility,
            type_annotation,
            default,
            hooks,
            span: Span {
                start: start_pos,
                end: last_end,
            },
        })
    }
    fn parse_trait_use(&mut self) -> Option<ClassMember> {
        debug_assert!(self.at(TokenKind::KwUse));
        let use_tok = self.bump();
        let start_pos = use_tok.span.start;

        let mut traits = Vec::new();
        let mut last_end = use_tok.span.end;

        loop {
            if let Some(qn) = self.parse_qualified_name() {
                last_end = qn.span.end;
                traits.push(qn);
            } else {
                break;
            }

            if self.eat(TokenKind::Comma) {
                continue;
            }
            break;
        }

        if self.eat(TokenKind::LBrace) {
            // Trait adaptations (insteadof/as)
            // For now, we just skip them until we have AST support
            let mut depth = 1;
            while !self.eof() && depth > 0 {
                if self.at(TokenKind::LBrace) {
                    depth += 1;
                } else if self.at(TokenKind::RBrace) {
                    depth -= 1;
                }
                let t = self.bump();
                last_end = t.span.end;
            }
        } else {
            self.expect_or_err(TokenKind::Semicolon, &mut last_end);
        }

        Some(ClassMember::TraitUse {
            traits,
            span: Span {
                start: start_pos,
                end: last_end,
            },
        })
    }

    fn at_set_visibility_clause(&self) -> bool {
        matches!(
            self.kind(),
            TokenKind::KwPublic | TokenKind::KwProtected | TokenKind::KwPrivate
        ) && *self.nth_kind(1) == TokenKind::LParen
            && *self.nth_kind(2) == TokenKind::KwSet
            && *self.nth_kind(3) == TokenKind::RParen
    }

    fn bump_set_visibility_clause(&mut self) -> Visibility {
        let visibility = match self.kind() {
            TokenKind::KwPublic => Visibility::Public,
            TokenKind::KwProtected => Visibility::Protected,
            TokenKind::KwPrivate => Visibility::Private,
            _ => unreachable!(),
        };

        let _visibility_token = self.bump();
        let _lparen_token = self.bump();
        let _set_token = self.bump();
        let _rparen_token = self.bump();

        visibility
    }

    fn parse_property_hooks(&mut self, last_end: &mut u32) -> PropertyHooks {
        debug_assert!(self.at(TokenKind::LBrace));
        let lb = self.bump();
        let start = lb.span.start;

        let mut get: Option<PropertyHookGet> = None;
        let mut set: Option<PropertyHookSet> = None;

        while !self.eof() && !self.at(TokenKind::RBrace) {
            match self.kind() {
                TokenKind::KwGet => {
                    let kw = self.bump();
                    let body = self.parse_hook_body(last_end);
                    get = Some(PropertyHookGet {
                        body,
                        span: Span {
                            start: kw.span.start,
                            end: *last_end,
                        },
                    });
                },
                TokenKind::KwSet => {
                    let kw = self.bump();

                    // optional: set(...)
                    let param = if self.eat(TokenKind::LParen) {
                        let p = if self.at(TokenKind::RParen) {
                            None
                        } else {
                            self.parse_single_param()
                        };
                        self.expect_or_err(TokenKind::RParen, last_end);
                        p
                    } else {
                        None
                    };

                    let body = self.parse_hook_body(last_end);

                    set = Some(PropertyHookSet {
                        param,
                        body,
                        span: Span {
                            start: kw.span.start,
                            end: *last_end,
                        },
                    });
                },
                TokenKind::Semicolon => {
                    self.bump();
                },
                _ => {
                    // recover: skip tokens until we hit get/set/} or ;
                    let err_pos = self.current_span().start;
                    self.error_and_recover(
                        Diagnostic::error_from_code(
                            ParseDiagnosticCode::ExpectedStatement,
                            Span::at(err_pos),
                        ),
                        &[
                            TokenKind::KwGet,
                            TokenKind::KwSet,
                            TokenKind::Semicolon,
                            TokenKind::RBrace,
                        ],
                    );
                },
            }
        }

        self.expect_or_err(TokenKind::RBrace, last_end);

        PropertyHooks {
            get,
            set,
            span: Span {
                start,
                end: *last_end,
            },
        }
    }

    fn parse_hook_body(&mut self, last_end: &mut u32) -> HookBody {
        if self.eat(TokenKind::FatArrow) {
            let expr = self.parse_expr_or_err(last_end);
            self.expect_or_err(TokenKind::Semicolon, last_end);
            HookBody::Expr(expr)
        } else if self.at(TokenKind::LBrace) {
            let lb = self.bump();
            if let Some((block, end)) =
                self.parse_block_after_lbrace(lb.span.start)
            {
                *last_end = end;
                HookBody::Block(block)
            } else {
                HookBody::Block(Block {
                    items: Vec::new(),
                    span: Span::at(lb.span.end),
                })
            }
        } else {
            let err_pos = self.current_span().start;
            self.error(Diagnostic::error_from_code(
                ParseDiagnosticCode::ExpectedExpression,
                Span::at(err_pos),
            ));
            HookBody::Expr(Expr::Error {
                span: Span::at(err_pos),
            })
        }
    }
}
