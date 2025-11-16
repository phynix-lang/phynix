use crate::ast::{Ident, Stmt, UseImport, UseKind};
use crate::parser::Parser;
use phynix_core::Span;
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_use_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwUse));

        let use_token = self.bump();
        let start_span = use_token.span;

        let mut import_kind = UseKind::Normal;
        if self.at(TokenKind::KwFunction) {
            let _function_token = self.bump();
            import_kind = UseKind::Function;
        } else if self.at(TokenKind::KwConst) {
            let _const_token = self.bump();
            import_kind = UseKind::Const;
        }

        let mut imports: Vec<UseImport> = Vec::new();

        loop {
            let qn_opt =
                self.parse_qualified_name("expected name in use statement");
            if qn_opt.is_none() {
                self.recover_to_any(&[TokenKind::Comma, TokenKind::Semicolon]);
                if self.eat(TokenKind::Comma) {
                    continue;
                }
                break;
            }

            let qn = qn_opt.unwrap();
            let mut last_end = qn.span.end;

            if self.at(TokenKind::LBrace) {
                let _lbrace_token = self.bump();

                loop {
                    let mut item_kind = import_kind;
                    if self.at(TokenKind::KwFunction) {
                        self.bump();
                        item_kind = UseKind::Function;
                    } else if self.at(TokenKind::KwConst) {
                        self.bump();
                        item_kind = UseKind::Const;
                    }

                    let first =
                        match self.expect_ident("expected name in group use") {
                            Some(token) => token,
                            None => {
                                self.error_and_recover(
                                    "expected name in group use",
                                    &[TokenKind::Comma, TokenKind::RBrace],
                                );
                                if self.eat(TokenKind::Comma) {
                                    continue;
                                }
                                break;
                            },
                        };

                    let mut tail_parts = vec![Ident { span: first.span }];
                    let mut tail_end = first.span.end;
                    self.extend_qn_parts_after_first(
                        &mut tail_parts,
                        &mut tail_end,
                    );

                    let alias_ident = self.parse_optional_alias(
                        &mut tail_end,
                        &[TokenKind::Comma, TokenKind::RBrace],
                    );

                    let mut full_parts: Vec<Ident> =
                        Vec::with_capacity(qn.parts.len() + tail_parts.len());

                    for part in &qn.parts {
                        full_parts.push(Ident { span: part.span });
                    }

                    for part in tail_parts {
                        full_parts.push(part);
                    }

                    let full_qn_span = Span {
                        start: qn.span.start,
                        end: tail_end,
                    };
                    let full_qn = crate::ast::QualifiedName {
                        absolute: qn.absolute,
                        parts: full_parts,
                        span: full_qn_span,
                    };

                    imports.push(UseImport {
                        kind: item_kind,
                        full_name: full_qn,
                        alias: alias_ident,
                        span: full_qn_span,
                    });

                    if self.eat(TokenKind::Comma) {
                        if !self.at(TokenKind::RBrace) {
                            continue;
                        }
                    }
                    break;
                }

                if let Some(rb) = self.expect(
                    TokenKind::RBrace,
                    "expected '}' to close group use",
                ) {
                    last_end = rb.span.end;
                } else {
                    self.error_and_recover(
                        "recovered after unterminated group use",
                        &[TokenKind::RBrace, TokenKind::Semicolon],
                    );
                }

                let end_span = if let Some(semi) = self.expect(
                    TokenKind::Semicolon,
                    "expected ';' after use statement",
                ) {
                    semi.span
                } else {
                    self.error_and_recover(
                        "recovered after unterminated use",
                        &[TokenKind::Semicolon, TokenKind::RBrace],
                    );
                    Span {
                        start: start_span.start,
                        end: last_end,
                    }
                };

                let full_span = Span {
                    start: start_span.start,
                    end: end_span.end,
                };
                return Some(Stmt::Use {
                    imports,
                    span: full_span,
                });
            } else {
                let alias_ident = self.parse_optional_alias(
                    &mut last_end,
                    &[TokenKind::Comma, TokenKind::Semicolon],
                );

                let import_span = Span {
                    start: qn.span.start,
                    end: last_end,
                };
                imports.push(UseImport {
                    kind: import_kind,
                    full_name: qn,
                    alias: alias_ident,
                    span: import_span,
                });

                if self.eat(TokenKind::Comma) {
                    continue;
                }
                break;
            }
        }

        let end_span = if let Some(semi_tok) = self
            .expect(TokenKind::Semicolon, "expected ';' after use statement")
        {
            semi_tok.span
        } else {
            self.error_and_recover(
                "recovered after unterminated use",
                &[
                    TokenKind::Semicolon,
                    TokenKind::RBrace,
                    TokenKind::Dollar,
                    TokenKind::Ident,
                    TokenKind::AttrOpen,
                ],
            );
            imports.last().map(|i| i.span).unwrap_or(start_span)
        };

        let full_span = Span {
            start: start_span.start,
            end: end_span.end,
        };

        Some(Stmt::Use {
            imports,
            span: full_span,
        })
    }

    fn parse_optional_alias(
        &mut self,
        end_ref: &mut u32,
        recover: &[TokenKind],
    ) -> Option<Ident> {
        if self.at(TokenKind::KwAs) {
            self.bump();
            if let Some(as_token) =
                self.expect_ident("expected alias after 'as'")
            {
                *end_ref = as_token.span.end;
                return Some(Ident {
                    span: as_token.span,
                });
            } else {
                self.error_and_recover("expected alias after 'as'", recover);
            }
        }
        None
    }
}
