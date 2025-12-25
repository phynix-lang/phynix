use crate::ast::{Attribute, AttributeArg, AttributeGroup, Ident};
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(crate) fn parse_attribute_group_list(
        &mut self,
    ) -> Option<Vec<AttributeGroup>> {
        if !self.at(TokenKind::AttrOpen) {
            return None;
        }

        let mut groups = Vec::new();

        while self.at(TokenKind::AttrOpen) {
            if let Some(group) = self.parse_single_attribute_group() {
                groups.push(group);
            } else {
                self.recover_to_any(&[
                    TokenKind::RBracket,
                    TokenKind::Semicolon,
                    TokenKind::RBrace,
                    TokenKind::Dollar,
                    TokenKind::AttrOpen,
                    TokenKind::Ident,
                ]);
                break;
            }
        }

        Some(groups)
    }

    fn parse_single_attribute_group(&mut self) -> Option<AttributeGroup> {
        debug_assert!(self.at(TokenKind::AttrOpen));

        let attr_open_token = self.bump();
        let start_span = attr_open_token.span;
        let mut attrs = Vec::new();

        loop {
            while self.at(TokenKind::Backslash) {
                self.bump();
            }

            if self.at(TokenKind::RBracket) {
                break;
            }

            let attr_start = self.current_span().start;

            let Some(name) =
                self.parse_qualified_name("expected attribute name")
            else {
                self.error_and_recover(
                    "expected attribute name",
                    &[TokenKind::Comma, TokenKind::RBracket],
                );
                if self.at(TokenKind::RBracket) {
                    break;
                }
                if self.eat(TokenKind::Comma) {
                    continue;
                }
                continue;
            };

            let mut args = Vec::new();

            if self.eat(TokenKind::LParen) {
                if !self.at(TokenKind::RParen) {
                    loop {
                        let arg_start = self.current_span().start;

                        if (self.at(TokenKind::Ident)
                            || self.is_ident_like_kw(self.kind()))
                            && self.at_nth(1, TokenKind::Colon)
                        {
                            let name_tok = self.expect_ident(
                                "expected identifier in named attribute argument",
                            );
                            let _colon = self.bump();

                            let Some(expr) = self.parse_expr() else {
                                self.error_and_recover(
                                    "expected expression after named-arg ':' in attribute",
                                    &[TokenKind::Comma, TokenKind::RParen],
                                );
                                if self.eat(TokenKind::Comma) {
                                    if self.at(TokenKind::RParen) {
                                        break;
                                    }
                                    continue;
                                }
                                break;
                            };

                            let arg_end = expr.span().end;
                            let name_tok = name_tok?;
                            let name_ident = Ident {
                                span: name_tok.span,
                            };

                            args.push(AttributeArg::Named {
                                name: name_ident,
                                expr,
                                span: Span {
                                    start: arg_start,
                                    end: arg_end,
                                },
                            });
                        } else {
                            let Some(expr) = self.parse_expr() else {
                                self.error_and_recover(
                                    "expected expression in attribute args",
                                    &[TokenKind::Comma, TokenKind::RParen],
                                );
                                if self.eat(TokenKind::Comma) {
                                    if self.at(TokenKind::RParen) {
                                        break;
                                    }
                                    continue;
                                }
                                break;
                            };

                            let arg_end = expr.span().end;
                            args.push(AttributeArg::Positional {
                                expr,
                                span: Span {
                                    start: arg_start,
                                    end: arg_end,
                                },
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

                let _ = self.expect(
                    TokenKind::RParen,
                    "expected ')' in attribute args",
                );
            }

            let attr_end =
                self.prev_span().map(|s| s.end).unwrap_or(attr_start);
            attrs.push(Attribute {
                name,
                args,
                span: Span {
                    start: attr_start,
                    end: attr_end,
                },
            });

            if self.eat(TokenKind::LParen) {
                if !self.at(TokenKind::RParen) {
                    loop {
                        if (self.at(TokenKind::Ident)
                            || self.is_ident_like_kw(self.kind()))
                            && self.at_nth(1, TokenKind::Colon)
                        {
                            let _name = self.bump();
                            let _colon = self.bump();
                            if self.parse_expr().is_none() {
                                self.error_and_recover(
                                    "expected expression after named-arg ':' in attribute",
                                    &[TokenKind::Comma, TokenKind::RParen],
                                );
                            }
                        } else {
                            if self.parse_expr().is_none() {
                                self.error_and_recover(
                                    "expected expression in attribute args",
                                    &[TokenKind::Comma, TokenKind::RParen],
                                );
                            }
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

                let _ = self.expect(
                    TokenKind::RParen,
                    "expected ')' in attribute args",
                );
            }

            if self.eat(TokenKind::Comma) {
                if self.at(TokenKind::RBracket) {
                    break;
                }
                continue;
            }

            if self.at(TokenKind::RBracket) {
                break;
            }

            self.error_and_recover(
                "expected ',' or ']' after attribute",
                &[TokenKind::Comma, TokenKind::RBracket],
            );
            if self.at(TokenKind::RBracket) {
                break;
            }
        }

        let end_span = if let Some(close_token) = self.expect(
            TokenKind::RBracket,
            "expected ']' to close attribute group",
        ) {
            close_token.span
        } else {
            start_span
        };

        let full_span = Span {
            start: start_span.start,
            end: end_span.end,
        };

        Some(AttributeGroup {
            attrs,
            span: full_span,
        })
    }
}
