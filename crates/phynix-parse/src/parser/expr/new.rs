use crate::ast::{Expr, Ident};
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_new_expr(&mut self) -> Option<Expr> {
        debug_assert!(self.at(TokenKind::KwNew));

        let new_token = self.bump();
        let start_pos = new_token.span.start;

        if self.at(TokenKind::KwClass) {
            let anon_span = self.skip_anonymous_class_after_new();
            return Some(Expr::Error {
                span: Span {
                    start: start_pos,
                    end: anon_span.end,
                },
            });
        }

        let class_expr = if self
            .at_any(&[TokenKind::Backslash, TokenKind::Ident])
        {
            if let Some(qn) =
                self.parse_qualified_name("expected name after 'new'")
            {
                let class_span = qn.span;
                Expr::VarRef {
                    name: Ident { span: qn.span },
                    span: class_span,
                }
            } else {
                self.error_and_recover(
                    "expected class name after 'new'",
                    &[
                        TokenKind::LParen,
                        TokenKind::Semicolon,
                        TokenKind::Comma,
                        TokenKind::RParen,
                        TokenKind::RBrace,
                    ],
                );
                let fake_span = self.prev_span().unwrap_or(new_token.span);
                Expr::Error { span: fake_span }
            }
        } else if self.at_variable_start() {
            match self.parse_variable_expr() {
                Some(dyn_expr) => dyn_expr,
                None => {
                    self.error_and_recover(
                        "expected variable after 'new $'",
                        &[
                            TokenKind::LParen,
                            TokenKind::Semicolon,
                            TokenKind::Comma,
                            TokenKind::RParen,
                            TokenKind::RBrace,
                        ],
                    );
                    let fake_span = self.prev_span().unwrap_or(new_token.span);
                    Expr::Error { span: fake_span }
                },
            }
        } else {
            self.error_and_recover(
                "expected class name after 'new'",
                &[
                    TokenKind::LParen,
                    TokenKind::Semicolon,
                    TokenKind::Comma,
                    TokenKind::RParen,
                    TokenKind::RBrace,
                ],
            );
            if self.eat(TokenKind::LParen) {
                let _ = self.skip_balanced_parens();
            }
            let fake_span = self.prev_span().unwrap_or(new_token.span);
            Expr::Error { span: fake_span }
        };

        let (args, end_pos) = if self.eat(TokenKind::LParen) {
            let lparen_span = self.prev_span().unwrap();
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

    fn skip_anonymous_class_after_new(&mut self) -> Span {
        debug_assert!(self.at(TokenKind::KwClass));
        let class_token = self.bump();
        let mut end = class_token.span.end;

        if self.eat(TokenKind::LParen) {
            end = self.skip_balanced_parens().end;
        }

        if self.eat(TokenKind::KwExtends) {
            end = self.skip_qualified_name_list(1).end;
        }

        if self.eat(TokenKind::KwImplements) {
            end = self.skip_qualified_name_list(usize::MAX).end;
        }

        if self.eat(TokenKind::LBrace) {
            end = self.skip_balanced_braces().end;
        } else {
            self.error_and_recover(
                "expected '{' to start anonymous class body",
                &[
                    TokenKind::LBrace,
                    TokenKind::Semicolon,
                    TokenKind::Comma,
                    TokenKind::RParen,
                    TokenKind::RBrace,
                ],
            );
            if self.eat(TokenKind::LBrace) {
                end = self.skip_balanced_braces().end;
            }
        }

        Span {
            start: class_token.span.start,
            end,
        }
    }

    fn skip_qualified_name_list(&mut self, max: usize) -> Span {
        let mut consumed = 0usize;
        let mut end = self.prev_span().map(|s| s.end).unwrap_or_default();

        while consumed < max {
            if let Some(name_span) = self.try_skip_qualified_name() {
                end = name_span.end;
                consumed += 1;
                if !self.eat(TokenKind::Comma) {
                    break;
                }
                end = self.prev_span().unwrap().end;
            } else {
                if consumed == 0 {
                    self.error_here("expected class/interface name");
                }
                break;
            }
        }

        Span {
            start: self.prev_span().map(|s| s.start).unwrap_or(end),
            end,
        }
    }

    fn try_skip_qualified_name(&mut self) -> Option<Span> {
        if !self.at(TokenKind::Ident) {
            return None;
        }

        let mut end_opt: Option<u32>;

        let first = self.bump();
        end_opt = Some(first.span.end);
        loop {
            if self.eat(TokenKind::Backslash) {
                if self.at(TokenKind::Ident) {
                    let id = self.bump();
                    end_opt = Some(id.span.end);
                    continue;
                }
                break;
            }
            break;
        }

        Some(Span {
            start: first.span.start,
            end: end_opt.unwrap(),
        })
    }

    fn skip_balanced_core(
        &mut self,
        open: TokenKind,
        close: TokenKind,
    ) -> Span {
        let mut depth = 1;
        let mut end = self.prev_span().unwrap().end;

        while depth > 0 && !self.at(TokenKind::Eof) {
            if self.eat(open) {
                depth += 1;
                continue;
            }
            if self.at(close) {
                let r = self.bump();
                end = r.span.end;
                depth -= 1;
                continue;
            }
            let _ = self.bump();
            end = self.prev_span().unwrap().end;
        }

        Span {
            start: self.prev_span().unwrap().start,
            end,
        }
    }

    fn skip_balanced_parens(&mut self) -> Span {
        self.skip_balanced_core(TokenKind::LParen, TokenKind::RParen)
    }

    fn skip_balanced_braces(&mut self) -> Span {
        self.skip_balanced_core(TokenKind::LBrace, TokenKind::RBrace)
    }
}
