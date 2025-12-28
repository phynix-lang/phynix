use crate::ast::{Block, ClosureUse, Expr, Ident};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

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
            let use_token = self.bump();
            last_end = use_token.span.end;

            if !self.at(TokenKind::LParen) {
                self.error_and_recover(
                    Diagnostic::error_from_code(
                        ParseDiagnosticCode::expected_token(TokenKind::LParen),
                        Span::at(last_end),
                    ),
                    &[
                        TokenKind::LParen,
                        TokenKind::RParen,
                        TokenKind::LBrace,
                        TokenKind::FatArrow,
                    ],
                );
            } else {
                let lparen = self.bump();
                last_end = lparen.span.end;

                loop {
                    if self.at(TokenKind::RParen) {
                        break;
                    }

                    let by_ref = self.eat(TokenKind::Amp);

                    let (name_span, full_span) = if self.at(TokenKind::VarIdent)
                    {
                        let var_token = self.bump();
                        last_end = var_token.span.end;
                        (var_token.span, var_token.span)
                    } else if self.at(TokenKind::Dollar) {
                        let dollar_tok = self.bump();
                        last_end = dollar_tok.span.end;

                        match self.expect_ident_or_err(&mut last_end) {
                            Some(name_tok) => {
                                let span = Span {
                                    start: dollar_tok.span.start,
                                    end: name_tok.span.end,
                                };
                                (name_tok.span, span)
                            },
                            None => {
                                self.error_and_recover(
                                    Diagnostic::error_from_code(
                                        ParseDiagnosticCode::ExpectedIdent,
                                        Span::at(last_end),
                                    ),
                                    &[TokenKind::Comma, TokenKind::RParen],
                                );
                                break;
                            },
                        }
                    } else {
                        self.error_and_recover(
                            Diagnostic::error_from_code(
                                ParseDiagnosticCode::expected_token(
                                    TokenKind::Dollar,
                                ),
                                Span::at(last_end),
                            ),
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

                if let Some(end) = self.expect_closing_or_recover(
                    TokenKind::RParen,
                    &[
                        TokenKind::LBrace,
                        TokenKind::FatArrow,
                        TokenKind::Semicolon,
                        TokenKind::RBrace,
                    ],
                ) {
                    last_end = end;
                }
            }
        }

        let (return_type, _new_last_end) =
            self.parse_optional_return_type(last_end);

        if self.at(TokenKind::LBrace) {
            let lbrace_tok = self.bump();
            let lbrace_start = lbrace_tok.span.start;
            last_end = lbrace_tok.span.end;

            let (body_block, body_end_pos) =
                match self.parse_block_after_lbrace(lbrace_start) {
                    Some(res) => res,
                    None => {
                        self.error_and_recover(
                            Diagnostic::error(
                                ParseDiagnosticCode::UnexpectedToken, // TODO: Better code
                                Span::at(last_end),
                                "invalid closure body",
                            ),
                            &[
                                TokenKind::RBrace,
                                TokenKind::Semicolon,
                                TokenKind::RParen,
                            ],
                        );

                        let dummy_span = Span::at(lbrace_start);

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
            let arrow_end = self.current_span().start;
            let body_expr = match self.parse_expr() {
                Some(expr) => expr,
                None => {
                    self.error_and_recover(
                        Diagnostic::error_from_code(
                            ParseDiagnosticCode::ExpectedExpression,
                            Span::at(arrow_end),
                        ),
                        &[
                            TokenKind::Semicolon,
                            TokenKind::Comma,
                            TokenKind::RBrace,
                            TokenKind::RParen,
                        ],
                    );

                    let fallback_span = Span::at(start_pos);

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
            Diagnostic::error_from_code(
                ParseDiagnosticCode::expected_token(TokenKind::LBrace),
                Span::at(last_end),
            ),
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
