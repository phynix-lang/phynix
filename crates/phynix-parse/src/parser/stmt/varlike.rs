use crate::ast::{Expr, Ident, Stmt, TypeRef};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

impl<'src> Parser<'src> {
    pub fn parse_varlike_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::Dollar));

        let save_pos = self.pos;
        let dollar_token = self.bump();
        let mut last_end = dollar_token.span.end;

        if self.at_variable_start() {
            self.pos = save_pos;
            return self.parse_expr_stmt();
        }

        let name_token = match self.expect_ident_or_err(&mut last_end) {
            Some(token) => {
                last_end = token.span.end;
                token
            },
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
            let colon_end = self.current_span().start;
            if let Some(qn) = self.parse_qualified_name() {
                let ty_span = qn.span;
                last_end = ty_span.end;
                maybe_type = Some(TypeRef::Named {
                    name: qn,
                    span: ty_span,
                });
            } else {
                self.error_and_recover(
                    Diagnostic::error_from_code(
                        ParseDiagnosticCode::ExpectedIdent,
                        Span::at(colon_end),
                    ),
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

            let end_span = if let Some(semi) = self.expect(TokenKind::Semicolon)
            {
                semi.span
            } else {
                self.error_and_recover(
                    Diagnostic::error(
                        ParseDiagnosticCode::UnexpectedToken, // TODO: Better code or message
                        Span::at(last_end),
                        "recovered after unterminated declaration",
                    ),
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

        let eq_token = self.bump();
        last_end = eq_token.span.end;

        let var_ident = Ident {
            span: name_token.span,
        };

        let right_hand_side_expr = if self.at(TokenKind::Semicolon) {
            self.error(Diagnostic::error_from_code(
                ParseDiagnosticCode::ExpectedExpression,
                Span::at(last_end),
            ));
            Expr::Error {
                span: Span::at(last_end),
            }
        } else {
            self.parse_expr_or_err(&mut last_end)
        };

        let end_span =
            if let Some(semi_token) = self.expect(TokenKind::Semicolon) {
                semi_token.span
            } else {
                self.error_and_recover(
                    Diagnostic::error(
                        ParseDiagnosticCode::UnexpectedToken, // TODO: Better code or message
                        Span::at(last_end),
                        "recovered after unterminated assignment",
                    ),
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
