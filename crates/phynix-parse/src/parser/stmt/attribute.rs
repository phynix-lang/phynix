use crate::ast::AttributeGroup;
use crate::parser::Parser;
use phynix_core::Span;
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_attribute_group_list(
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

        loop {
            while self.at(TokenKind::Backslash) {
                self.bump();
            }

            if !self.at(TokenKind::Ident) {
                self.error_here("expected attribute name");
                break;
            }

            let _name_token = self.bump();

            loop {
                if self.at(TokenKind::Backslash) {
                    self.bump();
                    if self.at(TokenKind::Ident) {
                        self.bump();
                        continue;
                    } else {
                        self.error_here(
                            "expected identifier after '\\' in attribute name",
                        );
                        break;
                    }
                } else {
                    break;
                }
            }

            if self.eat(TokenKind::LParen) {
                if !self.at(TokenKind::RParen) {
                    loop {
                        if self.at(TokenKind::Ident)
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

                let _rparen_token = self.expect(
                    TokenKind::RParen,
                    "expected ')' in attribute args",
                );
            }

            if self.eat(TokenKind::Comma) {
                continue;
            }

            break;
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

        Some(AttributeGroup { span: full_span })
    }
}
