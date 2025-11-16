use crate::ast::{Ident, Stmt, TypeRef};
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub fn parse_varlike_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::Dollar));

        let save_pos = self.pos;
        let dollar_token = self.bump();

        if self.at_variable_start() {
            self.pos = save_pos;
            return self.parse_expr_stmt();
        }

        let name_token =
            match self.expect_ident("expected variable name after '$'") {
                Some(token) => token,
                None => {
                    self.pos = save_pos;
                    return self.parse_expr_stmt();
                },
            };

        if !self.at_any(&[TokenKind::Colon, TokenKind::Eq]) {
            self.pos = save_pos;
            return self.parse_expr_stmt();
        }

        let mut maybe_type: Option<TypeRef> = None;
        if self.eat(TokenKind::Colon) {
            if let Some(qn) =
                self.parse_qualified_name("expected type after ':'")
            {
                let ty_span = qn.span;
                maybe_type = Some(TypeRef::Named {
                    name: qn,
                    span: ty_span,
                });
            } else {
                self.error_and_recover(
                    "expected type after ':'",
                    &[TokenKind::Eq, TokenKind::Semicolon],
                );
            }
        }

        if !self.at(TokenKind::Eq) {
            if maybe_type.is_none() {
                self.pos = save_pos;
                return self.parse_expr_stmt();
            }

            let var_ident = Ident {
                span: name_token.span,
            };

            let semi_token = self
                .expect(TokenKind::Semicolon, "expected ';' after declaration");
            let end_span = if let Some(semi) = semi_token.as_ref() {
                semi.span
            } else {
                self.error_and_recover(
                    "recovered after unterminated declaration",
                    &[
                        TokenKind::Semicolon,
                        TokenKind::RBrace,
                        TokenKind::Dollar,
                        TokenKind::Ident,
                        TokenKind::AttrOpen,
                    ],
                );
                name_token.span
            };

            let full_span = Span {
                start: dollar_token.span.start,
                end: end_span.end,
            };

            return Some(Stmt::VarDecl {
                name: var_ident,
                type_annotation: maybe_type,
                init: None,
                span: full_span,
            });
        }

        let _eq_token = self.bump();

        let var_ident = Ident {
            span: name_token.span,
        };

        let right_hand_side_expr = if let Some(expr) = self.parse_expr() {
            expr
        } else {
            self.error_and_recover(
                "expected expression after '='",
                &[
                    TokenKind::Semicolon,
                    TokenKind::RBrace,
                    TokenKind::Dollar,
                    TokenKind::Ident,
                    TokenKind::AttrOpen,
                ],
            );

            let end_span = if let Some(semi) = self
                .expect(TokenKind::Semicolon, "expected ';' after assignment")
                .as_ref()
            {
                semi.span
            } else {
                name_token.span
            };

            let full_span = Span {
                start: dollar_token.span.start,
                end: end_span.end,
            };

            return if maybe_type.is_some() {
                Some(Stmt::VarDecl {
                    name: var_ident,
                    type_annotation: maybe_type,
                    init: None,
                    span: full_span,
                })
            } else {
                self.pos = save_pos;
                self.parse_expr_stmt()
            };
        };

        let end_span = if let Some(semi_token) =
            self.expect(TokenKind::Semicolon, "expected ';' after assignment")
        {
            semi_token.span
        } else {
            self.error_and_recover(
                "recovered after unterminated assignment",
                &[
                    TokenKind::Semicolon,
                    TokenKind::RBrace,
                    TokenKind::Dollar,
                    TokenKind::Ident,
                    TokenKind::AttrOpen,
                ],
            );
            right_hand_side_expr.span()
        };

        let full_span = Span {
            start: dollar_token.span.start,
            end: end_span.end,
        };

        if maybe_type.is_some() {
            Some(Stmt::VarDecl {
                name: var_ident,
                type_annotation: maybe_type,
                init: Some(right_hand_side_expr),
                span: full_span,
            })
        } else {
            Some(Stmt::Assign {
                target: var_ident,
                value: right_hand_side_expr,
                span: full_span,
            })
        }
    }
}
