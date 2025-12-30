use crate::ast::{Expr, IncludeKind};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

impl<'src> Parser<'src> {
    pub(super) fn parse_include_expr(
        &mut self,
        kind: IncludeKind,
    ) -> Option<Expr> {
        let kw_token = self.bump();
        let start_pos = kw_token.span.start;

        let target_expr = if self.at(TokenKind::LParen) {
            let lp_tok = self.bump();
            let lp_end = lp_tok.span.end;
            let last_end = lp_end;
            let inner_expr = match self.parse_expr() {
                Some(mut expr) => {
                    loop {
                        if !self.at(TokenKind::Comma) {
                            break;
                        }
                        let comma_tok = self.bump();
                        let comma_end = comma_tok.span.end;

                        match self.parse_expr() {
                            Some(next) => {
                                expr = next;
                            },
                            None => {
                                self.error_and_recover(
                                    Diagnostic::error_from_code(
                                        ParseDiagnosticCode::ExpectedExpression,
                                        Span::at(comma_end),
                                    ),
                                    &[
                                        TokenKind::RParen,
                                        TokenKind::Semicolon,
                                        TokenKind::Comma,
                                        TokenKind::RBrace,
                                    ],
                                );
                                break;
                            },
                        }
                    }

                    expr
                },
                None => {
                    self.error_and_recover(
                        Diagnostic::error_from_code(
                            ParseDiagnosticCode::ExpectedExpression,
                            Span::at(last_end),
                        ),
                        &[
                            TokenKind::RParen,
                            TokenKind::Semicolon,
                            TokenKind::Comma,
                            TokenKind::RBrace,
                        ],
                    );

                    let fake_span = self.current_span();
                    let fake = Expr::Error { span: fake_span };

                    self.expect_closing_or_recover(
                        TokenKind::RParen,
                        &[
                            TokenKind::Semicolon,
                            TokenKind::Comma,
                            TokenKind::RBrace,
                            TokenKind::RParen,
                        ],
                    );

                    fake
                },
            };

            return if self.at(TokenKind::RParen) {
                let rp_end = self.bump().span.end;

                Some(Expr::Include {
                    kind,
                    target: Box::new(inner_expr),
                    span: Span {
                        start: start_pos,
                        end: rp_end,
                    },
                })
            } else {
                let sp = self.current_span();
                self.error_and_recover(
                    Diagnostic::error_from_code(
                        ParseDiagnosticCode::expected_token(TokenKind::RParen),
                        sp,
                    ),
                    &[
                        TokenKind::Semicolon,
                        TokenKind::Comma,
                        TokenKind::RBrace,
                        TokenKind::RParen,
                    ],
                );

                let fallback_end = inner_expr.span().end;
                Some(Expr::Include {
                    kind,
                    target: Box::new(inner_expr),
                    span: Span {
                        start: start_pos,
                        end: fallback_end,
                    },
                })
            };
        } else {
            let kw_end = kw_token.span.end;
            match self.parse_expr() {
                Some(expr) => expr,
                None => {
                    self.error_and_recover(
                        Diagnostic::error_from_code(
                            ParseDiagnosticCode::ExpectedExpression,
                            Span::at(kw_end),
                        ),
                        &[
                            TokenKind::Semicolon,
                            TokenKind::Comma,
                            TokenKind::RBrace,
                            TokenKind::RParen,
                        ],
                    );

                    Expr::Error {
                        span: self.current_span(),
                    }
                },
            }
        };

        let full_span = Span {
            start: start_pos,
            end: target_expr.span().end,
        };

        Some(Expr::Include {
            kind,
            target: Box::new(target_expr),
            span: full_span,
        })
    }

    pub(crate) fn keyword_to_include_kind(
        &self,
        kw: TokenKind,
    ) -> Option<IncludeKind> {
        match kw {
            TokenKind::KwInclude => Some(IncludeKind::Include),
            TokenKind::KwIncludeOnce => Some(IncludeKind::IncludeOnce),
            TokenKind::KwRequire => Some(IncludeKind::Require),
            TokenKind::KwRequireOnce => Some(IncludeKind::RequireOnce),
            _ => None,
        }
    }
}
