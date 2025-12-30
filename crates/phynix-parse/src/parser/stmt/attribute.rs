use crate::ast::{Attribute, AttributeArg, AttributeGroup, Ident};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

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
            let last_end = attr_start;

            let Some(name) = self.parse_qualified_name() else {
                self.error_and_recover(
                    Diagnostic::error_from_code(
                        ParseDiagnosticCode::ExpectedIdent,
                        Span::at(last_end),
                    ),
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

            if self.at(TokenKind::LParen) {
                let lp_tok = self.bump();
                let lp_end = lp_tok.span.end;
                if !self.at(TokenKind::RParen) {
                    loop {
                        let arg_start = self.current_span().start;

                        let (name, expr_err_span) = if (self
                            .at(TokenKind::Ident)
                            || self.is_ident_like_kw(self.kind()))
                            && self.at_nth(1, TokenKind::Colon)
                        {
                            let name_tok = self.expect_ident();
                            let colon = self.bump();
                            (Some(name_tok?), colon.span.end)
                        } else {
                            (None, arg_start.max(lp_end))
                        };

                        let Some(expr) = self.parse_expr() else {
                            self.error(Diagnostic::error_from_code(
                                ParseDiagnosticCode::ExpectedExpression,
                                Span::at(expr_err_span),
                            ));
                            if self.eat(TokenKind::Comma) {
                                if self.at(TokenKind::RParen) {
                                    break;
                                }
                                continue;
                            }
                            break;
                        };

                        let arg_end = expr.span().end;
                        let span = Span {
                            start: arg_start,
                            end: arg_end,
                        };

                        if let Some(name_tok) = name {
                            args.push(AttributeArg::Named {
                                name: Ident {
                                    span: name_tok.span,
                                },
                                expr,
                                span,
                            });
                        } else {
                            args.push(AttributeArg::Positional { expr, span });
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

                let mut rp_end = self.current_span().start;
                self.expect_or_err(TokenKind::RParen, &mut rp_end);
            }

            let attr_end =
                self.current_span().start.saturating_sub(1).max(attr_start);
            attrs.push(Attribute {
                name,
                args,
                span: Span {
                    start: attr_start,
                    end: attr_end,
                },
            });

            if self.eat(TokenKind::Comma) {
                if self.at(TokenKind::RBracket) {
                    break;
                }
                continue;
            }

            if self.at(TokenKind::RBracket) {
                break;
            }

            let err_pos = self.current_span().start;
            self.error_and_recover(
                Diagnostic::error_from_code(
                    ParseDiagnosticCode::expected_tokens([
                        TokenKind::Comma,
                        TokenKind::RBracket,
                    ]),
                    Span::at(err_pos),
                ),
                &[TokenKind::Comma, TokenKind::RBracket],
            );
            if self.at(TokenKind::RBracket) {
                break;
            }
        }

        let mut rb_end = self.current_span().start;
        let end = if self.expect_or_err(TokenKind::RBracket, &mut rb_end) {
            rb_end
        } else {
            start_span.end
        };

        let full_span = Span {
            start: start_span.start,
            end,
        };

        Some(AttributeGroup {
            attrs,
            span: full_span,
        })
    }
}
