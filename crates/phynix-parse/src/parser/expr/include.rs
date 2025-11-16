use crate::ast::{Expr, IncludeKind};
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_include_expr(
        &mut self,
        kind: IncludeKind,
    ) -> Option<Expr> {
        let kw_token = self.bump();
        let start_pos = kw_token.span.start;

        let target_expr = if self.eat(TokenKind::LParen) {
            let inner_expr = match self.parse_expr() {
                Some(mut expr) => {
                    loop {
                        if !self.eat(TokenKind::Comma) {
                            break;
                        }

                        match self.parse_expr() {
                            Some(next) => {
                                expr = next;
                            },
                            None => {
                                self.error_and_recover(
                                    "expected expression after ',' in include(...)",
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
                        "expected expression inside include(...)",
                        &[
                            TokenKind::RParen,
                            TokenKind::Semicolon,
                            TokenKind::Comma,
                            TokenKind::RBrace,
                        ],
                    );

                    let fake_span = self.prev_span().unwrap_or(kw_token.span);
                    let fake = Expr::Error { span: fake_span };

                    if self
                        .expect(
                            TokenKind::RParen,
                            "expected ')' after include(...)",
                        )
                        .is_none()
                    {
                        self.recover_to_any(&[
                            TokenKind::Semicolon,
                            TokenKind::Comma,
                            TokenKind::RBrace,
                            TokenKind::RParen,
                        ]);
                    }

                    fake
                },
            };

            return if let Some(rp_tok) = self
                .expect(TokenKind::RParen, "expected ')' after include(...)")
            {
                let full_span = Span {
                    start: start_pos,
                    end: rp_tok.span.end,
                };

                Some(Expr::Include {
                    kind,
                    target: Box::new(inner_expr),
                    span: full_span,
                })
            } else {
                self.recover_to_any(&[
                    TokenKind::Semicolon,
                    TokenKind::Comma,
                    TokenKind::RBrace,
                    TokenKind::RParen,
                ]);

                let fallback_end = inner_expr.span().end;
                let full_span = Span {
                    start: start_pos,
                    end: fallback_end,
                };

                Some(Expr::Include {
                    kind,
                    target: Box::new(inner_expr),
                    span: full_span,
                })
            };
        } else {
            match self.parse_expr() {
                Some(expr) => expr,
                None => {
                    self.error_and_recover(
                        "expected expression after include",
                        &[
                            TokenKind::Semicolon,
                            TokenKind::Comma,
                            TokenKind::RBrace,
                            TokenKind::RParen,
                        ],
                    );

                    let fake_span = self.prev_span().unwrap_or(kw_token.span);
                    Expr::Error { span: fake_span }
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
