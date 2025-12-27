use crate::ast::{Ident, Stmt};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_const_decl_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwConst));

        let const_token = self.bump();
        let start_pos = const_token.span.start;
        let mut last_end = const_token.span.end;

        let name_token =
            match self.expect_ident("expected constant name after 'const'") {
                Some(token) => {
                    last_end = token.span.end;
                    token
                },
                None => {
                    let fake_span = Span {
                        start: last_end,
                        end: last_end,
                    };

                    return Some(Stmt::ConstDecl {
                        name: Ident { span: fake_span },
                        value: None,
                        span: Span {
                            start: start_pos,
                            end: last_end,
                        },
                    });
                },
            };

        let const_ident = Ident {
            span: name_token.span,
        };

        if !self.eat(TokenKind::Eq) {
            self.error_here(
                ParseDiagnosticCode::ExpectedToken,
                "expected '=' after constant name",
            );

            let span = Span {
                start: start_pos,
                end: name_token.span.end,
            };

            return Some(Stmt::ConstDecl {
                name: const_ident,
                value: None,
                span,
            });
        }

        let value_expr = if let Some(expr) = self.parse_expr() {
            last_end = expr.span().end;
            Some(expr)
        } else {
            self.error_and_recover(
                "expected expression after '=' in const",
                &[
                    TokenKind::Semicolon,
                    TokenKind::RBrace,
                    TokenKind::AttrOpen,
                    TokenKind::Dollar,
                    TokenKind::Ident,
                ],
            );
            None
        };

        let end_span = if let Some(semi_tok) =
            self.expect(TokenKind::Semicolon, "expected ';' after const value")
        {
            semi_tok.span
        } else {
            Span {
                start: last_end,
                end: last_end,
            }
        };

        let full_span = Span {
            start: start_pos,
            end: end_span.end,
        };

        Some(Stmt::ConstDecl {
            name: const_ident,
            value: value_expr,
            span: full_span,
        })
    }
}
