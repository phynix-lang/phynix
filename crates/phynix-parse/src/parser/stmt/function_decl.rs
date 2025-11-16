use crate::ast::{Block, Ident, Param, QualifiedName, Stmt, TypeRef};
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_function_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwFunction));

        let fn_token = self.bump();
        let fn_start = fn_token.span.start;
        let mut last_end = fn_token.span.end;

        let _by_ref = self.eat(TokenKind::Amp);

        let (fn_ident, name_end) = if let Some(name_token) =
            self.expect_ident("expected function name after 'function'")
        {
            (
                Ident {
                    span: name_token.span,
                },
                name_token.span.end,
            )
        } else {
            let fake_span = Span {
                start: last_end,
                end: last_end,
            };
            (Ident { span: fake_span }, last_end)
        };
        last_end = name_end;

        let (params, _params_end) = match self.parse_param_list() {
            Some((params, end)) => {
                last_end = end;
                (params, end)
            },
            None => {
                self.error_and_recover(
                    "failed to parse parameter list",
                    &[
                        TokenKind::LBrace,
                        TokenKind::Semicolon,
                        TokenKind::RBrace,
                    ],
                );

                (Vec::new(), last_end)
            },
        };

        let (return_ty, new_last_end) =
            self.parse_optional_return_type(last_end);
        last_end = new_last_end;

        if !self.eat(TokenKind::LBrace) {
            self.recover_one_token("expected '{' to start function body");

            let body_block = Block {
                items: Vec::new(),
                span: Span {
                    start: last_end,
                    end: last_end,
                },
            };

            let span = Span {
                start: fn_start,
                end: last_end,
            };

            return Some(Stmt::Function {
                name: fn_ident,
                params,
                return_type: return_ty,
                body: body_block,
                span,
            });
        }

        let lbrace_span = self.prev_span().unwrap();
        let (body_block, body_end) =
            self.parse_block_after_lbrace(lbrace_span.start).unwrap();

        let fn_span = Span {
            start: fn_start,
            end: body_end,
        };

        Some(Stmt::Function {
            name: fn_ident,
            params,
            return_type: return_ty,
            body: body_block,
            span: fn_span,
        })
    }

    pub(crate) fn parse_param_list(&mut self) -> Option<(Vec<Param>, u32)> {
        let lparen_token =
            self.expect(TokenKind::LParen, "expected '(' after function name")?;

        let mut params = Vec::new();
        let mut last_end = lparen_token.span.end;

        if self.eat(TokenKind::RParen) {
            let rparen_end = self.prev_span().unwrap().end;
            return Some((params, rparen_end));
        }

        loop {
            if self.at(TokenKind::RParen) {
                break;
            }

            match self.parse_single_param() {
                Some(param) => {
                    last_end = param.span.end;
                    params.push(param);
                },
                None => {
                    self.error_and_recover(
                        "invalid parameter",
                        &[TokenKind::Comma, TokenKind::RParen],
                    );
                },
            }

            if self.eat(TokenKind::Comma) {
                if self.at(TokenKind::RParen) {
                    let r = self.bump();
                    last_end = r.span.end;
                    return Some((params, last_end));
                }
                continue;
            }

            break;
        }

        if self.at(TokenKind::RParen) {
            let rparen_token = self.bump();
            last_end = rparen_token.span.end;
        } else {
            self.error_and_recover(
                "expected ')' to close parameter list",
                &[TokenKind::RParen, TokenKind::LBrace, TokenKind::Semicolon],
            );
        }

        Some((params, last_end))
    }

    fn parse_single_param(&mut self) -> Option<Param> {
        let mut param_start = self.current_span().start;

        let _attrs = if self.at(TokenKind::AttrOpen) {
            let attr_start = self.current_span().start;
            let attrs = self.parse_attribute_group_list()?;
            param_start = attr_start;
            Some(attrs)
        } else {
            None
        };

        let (maybe_type, mut last_end) = self.parse_param_type_prefix();

        if self.eat(TokenKind::Amp) {
            last_end = self.prev_span().unwrap().end;
        }
        if self.eat(TokenKind::Ellipsis) {
            last_end = self.prev_span().unwrap().end;
        }

        let name_ident = if self.at(TokenKind::VarIdent) {
            let token = self.bump();
            last_end = token.span.end;
            Ident { span: token.span }
        } else {
            if !self.at(TokenKind::Dollar) {
                self.error_and_recover(
                    "expected '$' in parameter",
                    &[TokenKind::Comma, TokenKind::RParen],
                );
                return None;
            }
            let _dollar = self.bump();
            match self.expect_ident("expected parameter name after '$'") {
                Some(token) => {
                    last_end = token.span.end;
                    Ident { span: token.span }
                },
                None => {
                    self.error_and_recover(
                        "missing parameter name",
                        &[TokenKind::Comma, TokenKind::RParen],
                    );
                    let fake = Span {
                        start: last_end,
                        end: last_end,
                    };
                    return Some(Param {
                        name: Ident { span: fake },
                        type_annotation: maybe_type,
                        default: None,
                        span: Span {
                            start: param_start,
                            end: last_end,
                        },
                    });
                },
            }
        };

        let mut default_expr = None;
        if self.eat(TokenKind::Eq) {
            if let Some(expr) = self.parse_expr() {
                last_end = expr.span().end;
                default_expr = Some(expr);
            } else {
                self.error_and_recover(
                    "expected default expression after '=' in parameter",
                    &[TokenKind::Comma, TokenKind::RParen],
                );
            }
        }

        let span = Span {
            start: param_start,
            end: last_end,
        };

        Some(Param {
            name: name_ident,
            type_annotation: maybe_type,
            default: default_expr,
            span,
        })
    }

    pub(crate) fn parse_block_after_lbrace(
        &mut self,
        lbrace_start: u32,
    ) -> Option<(Block, u32)> {
        let mut items = Vec::new();
        let mut depth = 1;
        let mut body_end = lbrace_start;

        while !self.eof() && depth > 0 {
            if self.at(TokenKind::RBrace) {
                let rbrace = self.bump();
                depth -= 1;
                body_end = rbrace.span.end;
                continue;
            }

            let before = self.pos;
            if let Some(stmt) = self.parse_stmt() {
                if stmt.span().end > body_end {
                    body_end = stmt.span().end;
                }
                items.push(stmt);
                continue;
            }

            if self.pos == before {
                self.recover_stmt_in_block();
                if let Some(span) = self.prev_span() {
                    body_end = span.end;
                }
            }
        }

        let block_span = Span {
            start: lbrace_start,
            end: body_end,
        };

        let block = Block {
            items,
            span: block_span,
        };

        Some((block, body_end))
    }

    fn recover_stmt_in_block(&mut self) {
        let start_pos = self.pos;

        const MEMBER_KEYS: &[TokenKind] = &[
            TokenKind::KwPublic,
            TokenKind::KwPrivate,
            TokenKind::KwProtected,
            TokenKind::KwStatic,
            TokenKind::KwAbstract,
            TokenKind::KwFinal,
            TokenKind::KwReadonly,
            TokenKind::KwFunction,
            TokenKind::KwConst,
            TokenKind::KwUse,
            TokenKind::KwTrait,
            TokenKind::KwInterface,
            TokenKind::KwClass,
        ];

        const STMT_KEYS: &[TokenKind] = &[
            TokenKind::KwDeclare,
            TokenKind::KwDo,
            TokenKind::KwEcho,
            TokenKind::KwFn,
            TokenKind::KwFor,
            TokenKind::KwForeach,
            TokenKind::KwIf,
            TokenKind::KwInclude,
            TokenKind::KwIncludeOnce,
            TokenKind::KwNamespace,
            TokenKind::KwNew,
            TokenKind::KwRequire,
            TokenKind::KwRequireOnce,
            TokenKind::KwReturn,
            TokenKind::KwSwitch,
            TokenKind::KwThrow,
            TokenKind::KwTry,
            TokenKind::KwWhile,
        ];

        while !self.eof() {
            if self.at(TokenKind::RBrace) {
                break;
            }

            if self.eat(TokenKind::Semicolon) {
                break;
            }

            let k = *self.nth_kind(0);
            if MEMBER_KEYS.contains(&k) || STMT_KEYS.contains(&k) {
                break;
            }
            self.bump();
        }

        if self.pos == start_pos && !self.eof() {
            self.bump();
        }
    }

    fn parse_param_type_prefix(&mut self) -> (Option<TypeRef>, u32) {
        let mut saw_question = false;
        let mut start_pos = self.current_span().start;

        if self.eat(TokenKind::Question) {
            saw_question = true;
            start_pos = self.prev_span().unwrap().start;
        }

        let k0 = *self.nth_kind(0);
        let looks_like_type = matches!(
            k0,
            TokenKind::Ident | TokenKind::Backslash | TokenKind::KwArray
        );

        if !looks_like_type {
            let here = self.current_span().start;
            return (None, here);
        }

        let first_qn = if self.at(TokenKind::KwArray) {
            let token = self.bump();
            let span = token.span;
            QualifiedName {
                absolute: false,
                parts: vec![Ident { span }],
                span,
            }
        } else {
            match self
                .parse_qualified_name("expected type before parameter name")
            {
                Some(qn) => qn,
                None => {
                    let here = self.current_span().start;
                    return (None, here);
                },
            }
        };

        let first_span = first_qn.span;
        let mut last_end = first_span.end;

        let mut types: Vec<TypeRef> = vec![TypeRef::Named {
            name: first_qn,
            span: first_span,
        }];
        let (saw_union, saw_inter) =
            self.collect_additional_types(&mut types, &mut last_end);

        let mut ty = if types.len() == 1 {
            let token = types.pop().unwrap();
            last_end = token.span().end;
            token
        } else if saw_inter && !saw_union {
            TypeRef::Intersection {
                types,
                span: Span {
                    start: start_pos,
                    end: last_end,
                },
            }
        } else {
            TypeRef::Union {
                types,
                span: Span {
                    start: start_pos,
                    end: last_end,
                },
            }
        };

        if saw_question {
            ty = TypeRef::Nullable {
                inner: Box::new(ty),
                span: Span {
                    start: start_pos,
                    end: last_end,
                },
            };
        }

        (Some(ty), last_end)
    }

    pub(crate) fn parse_optional_return_type(
        &mut self,
        last_end_in: u32,
    ) -> (Option<TypeRef>, u32) {
        let mut last_end = last_end_in;

        if !self.eat(TokenKind::Colon) {
            return (None, last_end);
        }

        let mut saw_question = false;
        if self.eat(TokenKind::Question) {
            saw_question = true;
            last_end = self.prev_span().unwrap().end;
        }

        let first_qn = if self.at(TokenKind::KwArray) {
            let token = self.bump();
            let span = token.span;
            QualifiedName {
                absolute: false,
                parts: vec![Ident { span }],
                span,
            }
        } else {
            match self.parse_qualified_name("expected return type after ':'") {
                Some(qn) => qn,
                None => return (None, last_end),
            }
        };

        let first_span = first_qn.span;
        last_end = first_span.end;

        let mut types: Vec<TypeRef> = vec![TypeRef::Named {
            name: first_qn,
            span: first_span,
        }];
        let (saw_union, saw_inter) =
            self.collect_additional_types(&mut types, &mut last_end);

        let mut ty = if types.len() == 1 {
            let token = types.pop().unwrap();
            last_end = token.span().end;
            token
        } else if saw_inter && !saw_union {
            let first_start = types.first().unwrap().span().start;
            TypeRef::Intersection {
                types,
                span: Span {
                    start: first_start,
                    end: last_end,
                },
            }
        } else {
            let first_start = types.first().unwrap().span().start;
            TypeRef::Union {
                types,
                span: Span {
                    start: first_start,
                    end: last_end,
                },
            }
        };

        if saw_question {
            ty = TypeRef::Nullable {
                inner: Box::new(ty),
                span: Span {
                    start: last_end_in,
                    end: last_end,
                },
            };
        }

        (Some(ty), last_end)
    }

    fn collect_additional_types(
        &mut self,
        types: &mut Vec<TypeRef>,
        last_end: &mut u32,
    ) -> (bool, bool) {
        let mut saw_union = false;
        let mut saw_inter = false;

        let mut push_qn = |qn: QualifiedName| {
            let span = qn.span;
            *last_end = span.end;
            types.push(TypeRef::Named { name: qn, span });
        };

        loop {
            let before = self.pos;

            if self.eat(TokenKind::Pipe) {
                let next_qn = if self.at(TokenKind::KwArray) {
                    let token = self.bump();
                    let span = token.span;
                    Some(QualifiedName {
                        absolute: false,
                        parts: vec![Ident { span }],
                        span,
                    })
                } else {
                    self.parse_qualified_name("expected type after '|'")
                };

                if let Some(next_qn) = next_qn {
                    push_qn(next_qn);
                    saw_union = true;
                } else {
                    break;
                }
            } else if matches!(self.nth_kind(0), TokenKind::Amp)
                && matches!(
                    self.nth_kind(1),
                    TokenKind::Ident
                        | TokenKind::Backslash
                        | TokenKind::KwArray
                )
            {
                let _amp_token = self.bump();
                let qn = if self.at(TokenKind::KwArray) {
                    let token = self.bump();
                    let span = token.span;
                    Some(QualifiedName {
                        absolute: false,
                        parts: vec![Ident { span }],
                        span,
                    })
                } else {
                    self.parse_qualified_name("expected type after '&'")
                };

                if let Some(qn) = qn {
                    push_qn(qn);
                    saw_inter = true;
                } else {
                    break;
                }
            } else {
                break;
            }

            if self.pos == before {
                break;
            }
        }

        (saw_union, saw_inter)
    }
}
