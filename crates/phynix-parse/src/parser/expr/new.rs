use crate::ast::{Expr, Ident, QualifiedName};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

impl<'src> Parser<'src> {
    pub(super) fn parse_new_expr(&mut self) -> Option<Expr> {
        debug_assert!(self.at(TokenKind::KwNew));

        let new_token = self.bump();
        let start_pos = new_token.span.start;
        let mut last_end = new_token.span.end;

        if self.at(TokenKind::KwClass) {
            let anon_span = self.skip_anonymous_class_after_new(&mut last_end);
            return Some(Expr::Error {
                span: Span {
                    start: start_pos,
                    end: anon_span.end,
                },
            });
        }

        let class_expr =
            if self.at_any(&[TokenKind::Backslash, TokenKind::Ident]) {
                if let Some(qn) = self.parse_qualified_name() {
                    let class_span = qn.span;
                    Expr::ConstFetch {
                        name: qn,
                        span: class_span,
                    }
                } else {
                    self.error_and_recover(
                        Diagnostic::error_from_code(
                            ParseDiagnosticCode::ExpectedIdent,
                            Span::at(last_end),
                        ),
                        &[
                            TokenKind::LParen,
                            TokenKind::Semicolon,
                            TokenKind::Comma,
                            TokenKind::RParen,
                            TokenKind::RBrace,
                        ],
                    );
                    Expr::Error {
                        span: Span::at(last_end),
                    }
                }
            } else if self.at_any(&[
                TokenKind::KwSelf,
                TokenKind::KwParent,
                TokenKind::KwStatic,
            ]) {
                let tok = self.bump();
                let span = tok.span;
                Expr::ConstFetch {
                    name: QualifiedName {
                        absolute: false,
                        parts: vec![Ident { span }],
                        span,
                    },
                    span,
                }
            } else if self.at_variable_start() {
                self.parse_variable_expr()
            } else {
                self.error_and_recover(
                    Diagnostic::error_from_code(
                        ParseDiagnosticCode::ExpectedIdent,
                        Span::at(last_end),
                    ),
                    &[
                        TokenKind::LParen,
                        TokenKind::Semicolon,
                        TokenKind::Comma,
                        TokenKind::RParen,
                        TokenKind::RBrace,
                    ],
                );
                if self.eat(TokenKind::LParen) {
                    let skip_span = self.skip_balanced_parens(last_end);
                    last_end = skip_span.end;
                }
                Expr::Error {
                    span: Span::at(last_end),
                }
            };

        let (args, end_pos) = if let Some(lp) = self.expect(TokenKind::LParen) {
            let lparen_span = lp.span;
            let (args_vec, rparen_span) =
                self.parse_call_arguments(lparen_span);
            (args_vec, rparen_span.end)
        } else {
            (Vec::new(), class_expr.span().end)
        };

        let span = Span {
            start: start_pos,
            end: end_pos,
        };

        Some(Expr::New {
            class: Box::new(class_expr),
            args,
            span,
        })
    }

    fn skip_anonymous_class_after_new(&mut self, last_end: &mut u32) -> Span {
        debug_assert!(self.at(TokenKind::KwClass));
        let class_token = self.bump();
        *last_end = class_token.span.end;

        if self.eat(TokenKind::LParen) {
            *last_end = self.skip_balanced_parens(*last_end).end;
        }

        if self.eat(TokenKind::KwExtends) {
            *last_end = self.skip_qualified_name_list(1, *last_end).end;
        }

        if self.eat(TokenKind::KwImplements) {
            *last_end =
                self.skip_qualified_name_list(usize::MAX, *last_end).end;
        }

        if self.eat(TokenKind::LBrace) {
            *last_end = self.skip_balanced_braces(*last_end).end;
        } else {
            self.error_and_recover(
                Diagnostic::error_from_code(
                    ParseDiagnosticCode::expected_token(TokenKind::LBrace),
                    Span::at(*last_end),
                ),
                &[
                    TokenKind::LBrace,
                    TokenKind::Semicolon,
                    TokenKind::Comma,
                    TokenKind::RParen,
                    TokenKind::RBrace,
                ],
            );
            if self.eat(TokenKind::LBrace) {
                *last_end = self.skip_balanced_braces(*last_end).end;
            }
        }

        Span {
            start: class_token.span.start,
            end: *last_end,
        }
    }

    fn skip_qualified_name_list(
        &mut self,
        max: usize,
        mut last_end: u32,
    ) -> Span {
        let mut consumed = 0usize;

        while consumed < max {
            if let Some(name_span) = self.try_skip_qualified_name() {
                last_end = name_span.end;
                consumed += 1;
                if let Some(comma) = self.expect(TokenKind::Comma) {
                    last_end = comma.span.end;
                } else {
                    break;
                }
            } else {
                if consumed == 0 {
                    self.error(Diagnostic::error_from_code(
                        ParseDiagnosticCode::ExpectedIdent,
                        Span::at(last_end),
                    ));
                }
                break;
            }
        }

        Span::at(last_end)
    }

    fn try_skip_qualified_name(&mut self) -> Option<Span> {
        if !self.at(TokenKind::Ident) {
            return None;
        }

        let first = self.bump();
        let mut end = first.span.end;
        loop {
            if self.eat(TokenKind::Backslash) {
                if self.at(TokenKind::Ident) {
                    let id = self.bump();
                    end = id.span.end;
                    continue;
                }
                break;
            }
            break;
        }

        Some(Span {
            start: first.span.start,
            end,
        })
    }

    fn skip_balanced_core(
        &mut self,
        open: TokenKind,
        close: TokenKind,
        mut last_end: u32,
    ) -> Span {
        let mut depth = 1;

        while depth > 0 && !self.at(TokenKind::Eof) {
            if self.eat(open) {
                depth += 1;
                continue;
            }
            if self.at(close) {
                let r = self.bump();
                last_end = r.span.end;
                depth -= 1;
                continue;
            }
            let tok = self.bump();
            last_end = tok.span.end;
        }

        Span::at(last_end)
    }

    fn skip_balanced_parens(&mut self, last_end: u32) -> Span {
        self.skip_balanced_core(TokenKind::LParen, TokenKind::RParen, last_end)
    }

    fn skip_balanced_braces(&mut self, last_end: u32) -> Span {
        self.skip_balanced_core(TokenKind::LBrace, TokenKind::RBrace, last_end)
    }
}
