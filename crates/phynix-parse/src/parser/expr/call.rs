use crate::ast::{Arg, ClassNameRef, Expr, Ident, QualifiedName};
use crate::parser::expr::SYNC_POSTFIX;
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::{Token, TokenKind};

impl<'src> Parser<'src> {
    pub(super) fn parse_identifier_expr(&mut self) -> Option<Expr> {
        debug_assert!(self.at(TokenKind::Ident));

        let first = self.bump();

        let mut parts: Vec<Ident> = vec![Ident { span: first.span }];
        let mut end = first.span.end;
        self.extend_qn_parts_after_first(&mut parts, &mut end);

        let qn_span = Span {
            start: first.span.start,
            end,
        };
        let qn = QualifiedName {
            absolute: false,
            parts,
            span: qn_span,
        };

        if self.eat(TokenKind::ColCol) {
            let class_name = ClassNameRef::Qualified(qn);

            if self.at(TokenKind::KwClass) {
                let class_token = self.bump();
                let span = Span {
                    start: qn_span.start,
                    end: class_token.span.end,
                };
                return Some(Expr::ClassConstFetch {
                    class_name,
                    constant: Ident {
                        span: class_token.span,
                    },
                    span,
                });
            }

            if self.at(TokenKind::VarIdent) {
                let var_token = self.bump();
                let span = Span {
                    start: qn_span.start,
                    end: var_token.span.end,
                };
                return Some(Expr::StaticPropertyFetch {
                    class_name,
                    property: Ident {
                        span: var_token.span,
                    },
                    span,
                });
            }

            if self.eat(TokenKind::Dollar) {
                if self.eat(TokenKind::LBrace) {
                    let inner = match self.parse_expr() {
                        Some(e) => e,
                        None => {
                            self.error_and_recover(
                                "expected expression after '::${'",
                                &[TokenKind::RBrace],
                            );
                            Expr::Error {
                                span: self.prev_span().unwrap_or(qn_span),
                            }
                        },
                    };
                    let _ = self.expect(
                        TokenKind::RBrace,
                        "expected '}' after '::${expr}'",
                    );
                    let span = Span {
                        start: qn_span.start,
                        end: inner.span().end,
                    };
                    return Some(Expr::StaticPropertyFetch {
                        class_name,
                        property: Ident { span: inner.span() },
                        span,
                    });
                }

                if let Some(prop_tok) =
                    self.expect_ident("expected property after '::$'")
                {
                    let span = Span {
                        start: qn_span.start,
                        end: prop_tok.span.end,
                    };
                    return Some(Expr::StaticPropertyFetch {
                        class_name,
                        property: Ident {
                            span: prop_tok.span,
                        },
                        span,
                    });
                }

                self.error_and_recover(
                    "expected property after '::$'",
                    SYNC_POSTFIX,
                );
                return Some(Expr::Error { span: qn_span });
            }

            if self.eat(TokenKind::LBrace) {
                let inner = match self.parse_expr() {
                    Some(e) => e,
                    None => {
                        self.error_and_recover(
                            "expected expression after '{' in '::{...}'",
                            &[TokenKind::RBrace],
                        );
                        Expr::Error {
                            span: self.prev_span().unwrap_or(qn_span),
                        }
                    },
                };

                let _ = self
                    .expect(TokenKind::RBrace, "expected '}' after '::{expr}'");

                let method_span = inner.span();
                return Some(self.build_static_named_member_or_call(
                    class_name,
                    method_span,
                    qn_span.start,
                ));
            }

            if let Some(id_tok) = self.expect_ident_like_or_sync(
                "expected identifier after '::'",
                SYNC_POSTFIX,
            ) {
                let id_span = id_tok.span;
                return Some(self.build_static_named_member_or_call(
                    class_name,
                    id_span,
                    qn_span.start,
                ));
            }

            self.error_and_recover(
                "expected identifier, 'class', or '$' after '::'",
                SYNC_POSTFIX,
            );
            return Some(Expr::Error { span: qn_span });
        }

        if self.eat(TokenKind::LParen) {
            let lparen_span = self.prev_span().unwrap();
            let (args, rparen_span) = self.parse_call_arguments(lparen_span);
            let span = Span {
                start: qn_span.start,
                end: rparen_span.end,
            };

            let name_ref = Expr::NameRef {
                name: QualifiedName {
                    absolute: qn.absolute,
                    parts: qn
                        .parts
                        .iter()
                        .map(|p| Ident { span: p.span })
                        .collect(),
                    span: qn_span,
                },
                span: qn_span,
            };

            return Some(Expr::FunctionCall {
                callee: Box::new(name_ref),
                args,
                span,
            });
        }

        Some(Expr::NameRef {
            name: qn,
            span: qn_span,
        })
    }

    #[inline]
    fn build_static_named_member_or_call(
        &mut self,
        class_name: ClassNameRef,
        method_span: Span,
        qn_start: u32,
    ) -> Expr {
        if self.eat(TokenKind::LParen) {
            let lp = self.prev_span().unwrap();
            let (args, rp) = self.parse_call_arguments(lp);
            let span = Span {
                start: qn_start,
                end: rp.end,
            };
            Expr::StaticCall {
                class: class_name,
                method: Ident { span: method_span },
                args,
                span,
            }
        } else {
            let span = Span {
                start: qn_start,
                end: method_span.end,
            };
            Expr::ClassConstFetch {
                class_name,
                constant: Ident { span: method_span },
                span,
            }
        }
    }

    #[inline]
    fn expect_ident_like_or_sync(
        &mut self,
        msg: &'static str,
        sync: &[TokenKind],
    ) -> Option<&'src Token> {
        if let Some(token) = self.bump_ident_like() {
            return Some(token);
        }
        self.error_and_recover(msg, sync);
        None
    }

    pub(super) fn parse_call_arguments(
        &mut self,
        lparen_span: Span,
    ) -> (Vec<Arg>, Span) {
        let mut args: Vec<Arg> = Vec::new();
        let mut saw_named = false;

        if !self.at(TokenKind::RParen) {
            loop {
                let arg_start = self.current_span().start;

                if self.at(TokenKind::Ellipsis) {
                    let ell = self.bump();

                    if args.is_empty()
                        && !saw_named
                        && self.at(TokenKind::RParen)
                    {
                        break;
                    }

                    if saw_named {
                        self.error_here(
                            "variadic unpack not allowed after named arguments",
                        );
                    }

                    if let Some(e) = self.parse_expr() {
                        let span = Span {
                            start: arg_start,
                            end: e.span().end,
                        };
                        args.push(Arg {
                            name: None,
                            unpack: true,
                            expr: e,
                            span,
                        });
                    } else {
                        self.error_and_recover(
                            "expected expression after '...'",
                            &[TokenKind::Comma, TokenKind::RParen],
                        );
                        let span = self.prev_span().unwrap_or(ell.span);
                        args.push(Arg {
                            name: None,
                            unpack: true,
                            expr: Expr::Error { span },
                            span,
                        });
                    }
                } else if self.at(TokenKind::Ident)
                    && self.at_nth(1, TokenKind::Colon)
                {
                    let name_token = self.bump();
                    let _colon = self.bump();
                    saw_named = true;

                    if let Some(e) = self.parse_expr() {
                        let span = Span {
                            start: arg_start,
                            end: e.span().end,
                        };
                        args.push(Arg {
                            name: Some(Ident {
                                span: name_token.span,
                            }),
                            unpack: false,
                            expr: e,
                            span,
                        });
                    } else {
                        self.error_and_recover(
                            "expected expression after named-arg ':'",
                            &[TokenKind::Comma, TokenKind::RParen],
                        );
                        let span = self.prev_span().unwrap_or(name_token.span);
                        args.push(Arg {
                            name: Some(Ident {
                                span: name_token.span,
                            }),
                            unpack: false,
                            expr: Expr::Error { span },
                            span,
                        });
                    }
                } else if let Some(e) = self.parse_expr() {
                    if saw_named {
                        self.error_here("positional argument not allowed after named arguments");
                    }
                    let span = Span {
                        start: arg_start,
                        end: e.span().end,
                    };
                    args.push(Arg {
                        name: None,
                        unpack: false,
                        expr: e,
                        span,
                    });
                } else {
                    self.error_and_recover(
                        "expected expression in argument list",
                        &[TokenKind::Comma, TokenKind::RParen],
                    );
                    let span = self.prev_span().unwrap_or(lparen_span);
                    args.push(Arg {
                        name: None,
                        unpack: false,
                        expr: Expr::Error { span },
                        span,
                    });
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

        let rparen_span = if let Some(rp_tok) =
            self.expect(TokenKind::RParen, "expected ')' after arguments")
        {
            rp_tok.span
        } else {
            self.recover_to_any(&[
                TokenKind::Semicolon,
                TokenKind::Comma,
                TokenKind::RParen,
                TokenKind::RBracket,
                TokenKind::RBrace,
            ]);
            lparen_span
        };

        (args, rparen_span)
    }

    pub(crate) fn extend_qn_parts_after_first(
        &mut self,
        parts: &mut Vec<Ident>,
        end: &mut u32,
    ) {
        while self.at(TokenKind::Backslash) {
            self.bump();
            if let Some(id) =
                self.expect_ident("expected identifier after '\\'")
            {
                *end = id.span.end;
                parts.push(Ident { span: id.span });
            } else {
                break;
            }
        }
    }
}
