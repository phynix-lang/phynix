use crate::ast::{Arg, ClassNameRef, Expr, Ident, QualifiedName};
use crate::parser::expr::SYNC_POSTFIX;
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

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

        if self.at(TokenKind::ColCol) {
            let colcol_end = self.current_span().end;
            self.bump();
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

            if self.at(TokenKind::Dollar) {
                let dollar_end = self.current_span().end;
                self.bump();
                if self.at(TokenKind::LBrace) {
                    let lbrace_end = self.current_span().end;
                    self.bump();

                    let inner = self.parse_braced_expr_or_error(
                        lbrace_end,
                        qn_span.start,
                        &[TokenKind::RBrace, TokenKind::Semicolon],
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

                if let Some(prop_tok) = self.expect_ident() {
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
                    Diagnostic::error_from_code(
                        ParseDiagnosticCode::ExpectedIdent,
                        Span::at(dollar_end),
                    ),
                    SYNC_POSTFIX,
                );
                return Some(Expr::Error { span: qn_span });
            }

            if self.at(TokenKind::LBrace) {
                let lbrace_end = self.current_span().end;
                self.bump();

                let inner = self.parse_braced_expr_or_error(
                    lbrace_end,
                    qn_span.start,
                    &[TokenKind::RBrace, TokenKind::Semicolon],
                );

                let method_span = inner.span();
                return Some(self.build_static_named_member_or_call(
                    class_name,
                    method_span,
                    qn_span.start,
                ));
            }

            if let Some(id_tok) = self.expect_ident() {
                let id_span = id_tok.span;
                return Some(self.build_static_named_member_or_call(
                    class_name,
                    id_span,
                    qn_span.start,
                ));
            }

            self.error_and_recover(
                Diagnostic::error_from_code(
                    ParseDiagnosticCode::expected_one_of([
                        ParseDiagnosticCode::ExpectedIdent,
                        ParseDiagnosticCode::expected_tokens([
                            TokenKind::KwClass,
                            TokenKind::Dollar,
                        ]),
                    ]),
                    Span::at(colcol_end),
                ),
                SYNC_POSTFIX,
            );
            return Some(Expr::Error { span: qn_span });
        }

        if self.at(TokenKind::LParen) {
            let lp_tok = self.bump();
            let lparen_span = lp_tok.span;
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
        if self.at(TokenKind::LParen) {
            let lp_tok = self.bump();
            let (args, rp) = self.parse_call_arguments(lp_tok.span);
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

    pub(super) fn parse_call_arguments(
        &mut self,
        lparen_span: Span,
    ) -> (Vec<Arg>, Span) {
        let mut args: Vec<Arg> = Vec::new();
        let mut saw_named = false;
        let mut had_error = false;

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
                        self.error(Diagnostic::error_from_code(
                            ParseDiagnosticCode::VariadicAfterNamedArg,
                            ell.span,
                        ));
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
                            Diagnostic::error_from_code(
                                ParseDiagnosticCode::ExpectedExpression,
                                Span::at(ell.span.end),
                            ),
                            &[
                                TokenKind::Comma,
                                TokenKind::RParen,
                                TokenKind::Semicolon,
                            ],
                        );
                        let _ = self.eat(TokenKind::RParen);
                        had_error = true;
                        let span = Span {
                            start: arg_start,
                            end: ell.span.end,
                        };
                        args.push(Arg {
                            name: None,
                            unpack: true,
                            expr: Expr::Error { span },
                            span,
                        });
                        break;
                    }
                } else if self.at(TokenKind::Ident)
                    && self.at_nth(1, TokenKind::Colon)
                {
                    let name_token = self.bump();
                    let colon = self.bump();
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
                            Diagnostic::error_from_code(
                                ParseDiagnosticCode::ExpectedExpression,
                                Span::at(colon.span.end),
                            ),
                            &[
                                TokenKind::Comma,
                                TokenKind::RParen,
                                TokenKind::Semicolon,
                            ],
                        );
                        let _ = self.eat(TokenKind::RParen);
                        had_error = true;
                        let span = Span {
                            start: arg_start,
                            end: colon.span.end,
                        };
                        args.push(Arg {
                            name: Some(Ident {
                                span: name_token.span,
                            }),
                            unpack: false,
                            expr: Expr::Error { span },
                            span,
                        });
                        break;
                    }
                } else if let Some(e) = self.parse_expr() {
                    if saw_named {
                        let sp = Span {
                            start: arg_start,
                            end: e.span().end,
                        };
                        self.error(Diagnostic::error_from_code(
                            ParseDiagnosticCode::PositionalAfterNamedArg,
                            sp,
                        ));
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
                        Diagnostic::error_from_code(
                            ParseDiagnosticCode::ExpectedExpression,
                            Span::at(arg_start),
                        ),
                        &[
                            TokenKind::Comma,
                            TokenKind::RParen,
                            TokenKind::Semicolon,
                        ],
                    );
                    let _ = self.eat(TokenKind::RParen);
                    had_error = true;
                    let span = Span::at(arg_start);
                    args.push(Arg {
                        name: None,
                        unpack: false,
                        expr: Expr::Error { span },
                        span,
                    });
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

        let rparen_span = if let Some(rp_tok) = self.expect(TokenKind::RParen) {
            rp_tok.span
        } else if had_error {
            lparen_span
        } else {
            let last_end =
                args.last().map(|a| a.span.end).unwrap_or(lparen_span.end);
            self.error_and_recover(
                Diagnostic::error_from_code(
                    ParseDiagnosticCode::expected_token(TokenKind::RParen),
                    Span::at(last_end),
                ),
                &[
                    TokenKind::Semicolon,
                    TokenKind::Comma,
                    TokenKind::RParen,
                    TokenKind::RBracket,
                    TokenKind::RBrace,
                ],
            );
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
            let backslash = self.bump();
            if let Some(id) = self.expect_ident_or_err(end) {
                *end = id.span.end;
                parts.push(Ident { span: id.span });
            } else {
                *end = backslash.span.end;
                break;
            }
        }
    }

    fn parse_braced_expr_or_error(
        &mut self,
        lbrace_end: u32,
        error_span_start: u32,
        sync: &[TokenKind],
    ) -> Expr {
        match self.parse_expr() {
            Some(e) => {
                let mut expr_end = e.span().end;
                self.expect_or_err(TokenKind::RBrace, &mut expr_end);
                e
            },
            None => {
                self.error_and_recover(
                    Diagnostic::error_from_code(
                        ParseDiagnosticCode::ExpectedExpression,
                        Span::at(lbrace_end),
                    ),
                    sync,
                );
                let _ = self.eat(TokenKind::RBrace);
                Expr::Error {
                    span: Span::at(error_span_start),
                }
            },
        }
    }
}
