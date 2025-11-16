use crate::ast::{Block, ClosureUse, Expr, Ident};
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_closure_expr(&mut self) -> Option<Expr> {
        debug_assert!(self.at_any(&[
            TokenKind::KwFn,
            TokenKind::KwFunction,
            TokenKind::KwStatic
        ]));

        let is_static = self.at(TokenKind::KwStatic) && {
            self.bump();
            true
        };

        let fn_token = self.bump();
        let start_pos = fn_token.span.start;

        let (params, mut last_end) = self.parse_param_list()?;

        let mut captured_uses: Vec<ClosureUse> = Vec::new();
        if self.at(TokenKind::KwUse) {
            let _use_token = self.bump();

            if !self.at(TokenKind::LParen) {
                self.error_and_recover(
                    "expected '(' after 'use'",
                    &[
                        TokenKind::LParen,
                        TokenKind::RParen,
                        TokenKind::LBrace,
                        TokenKind::FatArrow,
                    ],
                );
            } else {
                let _lparen = self.bump();

                loop {
                    if self.at(TokenKind::RParen) {
                        break;
                    }

                    let by_ref = self.eat(TokenKind::Amp);

                    let (name_span, full_span) = if self.at(TokenKind::VarIdent)
                    {
                        let var_token = self.bump();
                        (var_token.span, var_token.span)
                    } else if self.eat(TokenKind::Dollar) {
                        match self
                            .expect_ident("expected variable name in use()")
                        {
                            Some(name_tok) => {
                                let span = Span {
                                    start: self.prev_span().unwrap().start,
                                    end: name_tok.span.end,
                                };
                                (name_tok.span, span)
                            },
                            None => {
                                self.error_and_recover(
                                    "expected variable name in use()",
                                    &[TokenKind::Comma, TokenKind::RParen],
                                );
                                break;
                            },
                        }
                    } else {
                        self.error_and_recover(
                            "expected '$' in use()",
                            &[TokenKind::Comma, TokenKind::RParen],
                        );
                        break;
                    };

                    captured_uses.push(ClosureUse {
                        by_ref,
                        name: Ident { span: name_span },
                        span: full_span,
                    });

                    if self.eat(TokenKind::Comma) {
                        continue;
                    }
                    break;
                }

                if let Some(rp) = self
                    .expect(TokenKind::RParen, "expected ')' after use() list")
                {
                    last_end = rp.span.end;
                } else {
                    self.recover_to_any(&[
                        TokenKind::LBrace,
                        TokenKind::FatArrow,
                        TokenKind::Semicolon,
                        TokenKind::RBrace,
                    ]);
                }
            }
        }

        let (return_type, _new_last_end) =
            self.parse_optional_return_type(last_end);

        if self.eat(TokenKind::LBrace) {
            let lbrace_start = self.prev_span().unwrap().start;

            let (body_block, body_end_pos) =
                match self.parse_block_after_lbrace(lbrace_start) {
                    Some(res) => res,
                    None => {
                        self.error_and_recover(
                            "invalid closure body",
                            &[
                                TokenKind::RBrace,
                                TokenKind::Semicolon,
                                TokenKind::RParen,
                            ],
                        );

                        let dummy_span = Span {
                            start: lbrace_start,
                            end: lbrace_start,
                        };

                        let empty_block = Block {
                            items: Vec::new(),
                            span: dummy_span,
                        };

                        let full_span = Span {
                            start: start_pos,
                            end: dummy_span.end,
                        };

                        return Some(Expr::Closure {
                            is_static,
                            params,
                            uses: captured_uses,
                            return_type,
                            body: empty_block,
                            span: full_span,
                        });
                    },
                };

            last_end = body_end_pos;

            let full_span = Span {
                start: start_pos,
                end: last_end,
            };

            return Some(Expr::Closure {
                is_static,
                params,
                uses: captured_uses,
                return_type,
                body: body_block,
                span: full_span,
            });
        }

        if self.eat(TokenKind::FatArrow) {
            let body_expr = match self.parse_expr() {
                Some(expr) => expr,
                None => {
                    self.error_and_recover(
                        "expected expression after '=>' in arrow closure",
                        &[
                            TokenKind::Semicolon,
                            TokenKind::Comma,
                            TokenKind::RBrace,
                            TokenKind::RParen,
                        ],
                    );

                    let fallback_span = self.prev_span().unwrap_or(Span {
                        start: start_pos,
                        end: start_pos,
                    });

                    Expr::Error {
                        span: fallback_span,
                    }
                },
            };

            last_end = body_expr.span().end;

            let full_span = Span {
                start: start_pos,
                end: last_end,
            };

            return Some(Expr::ArrowClosure {
                is_static,
                params,
                uses: captured_uses,
                return_type,
                body: Box::new(body_expr),
                span: full_span,
            });
        }

        self.error_and_recover(
            "expected '{' or '=>' after closure signature",
            &[
                TokenKind::Semicolon,
                TokenKind::Comma,
                TokenKind::RBrace,
                TokenKind::RParen,
            ],
        );

        None
    }
}
