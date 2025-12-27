use crate::ast::{Block, Expr, Stmt};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_for_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwFor));

        let kw = self.bump();
        let start = kw.span.start;
        let mut last_end = kw.span.end;

        if let Some(lp) =
            self.expect(TokenKind::LParen, "expected '(' after 'for'")
        {
            last_end = lp.span.end;
        }

        let init = if !self.at(TokenKind::Semicolon) {
            self.parse_for_expr_list(&[TokenKind::Semicolon, TokenKind::RParen])
        } else {
            None
        };
        if let Some(semi) =
            self.expect(TokenKind::Semicolon, "expected ';' after for init")
        {
            last_end = semi.span.end;
        }

        let cond = if !self.at(TokenKind::Semicolon) {
            self.parse_for_expr_list(&[TokenKind::Semicolon, TokenKind::RParen])
        } else {
            None
        };
        if let Some(semi) = self
            .expect(TokenKind::Semicolon, "expected ';' after for condition")
        {
            last_end = semi.span.end;
        }

        let step = if !self.at(TokenKind::RParen) {
            self.parse_for_expr_list(&[TokenKind::RParen])
        } else {
            None
        };
        let rp =
            self.expect(TokenKind::RParen, "expected ')' after for header");
        if let Some(token) = rp.as_ref() {
            last_end = token.span.end;
        }

        if self.eat(TokenKind::Colon) {
            let (body, end) = self.parse_until_any(&[TokenKind::KwEndFor]);
            last_end = end;

            let end_token =
                self.expect(TokenKind::KwEndFor, "expected 'endfor'");
            if let Some(token) = end_token {
                last_end = token.span.end;
            }

            let _ = self.eat(TokenKind::Semicolon);

            let span = Span {
                start,
                end: last_end,
            };

            return Some(Stmt::For {
                init,
                cond,
                increment: step,
                body,
                span,
            });
        }

        let body = self.parse_stmt_block_or_single(start, &mut last_end);

        let span = Span {
            start,
            end: last_end,
        };

        Some(Stmt::For {
            init,
            cond,
            increment: step,
            body,
            span,
        })
    }

    fn parse_for_expr_list(
        &mut self,
        terminators: &[TokenKind],
    ) -> Option<Expr> {
        let mut last_expr = match self.parse_expr() {
            Some(e) => e,
            None => return None,
        };

        while self.eat(TokenKind::Comma) {
            if self.at_any(terminators) {
                self.error_here(
                    ParseDiagnosticCode::ExpectedExpression,
                    "expected expression after ',' in for clause",
                );
                break;
            }

            match self.parse_expr() {
                Some(e) => {
                    last_expr = e;
                },
                None => {
                    self.error_and_recover(
                        "expected expression after ',' in for clause",
                        terminators,
                    );
                    break;
                },
            }
        }

        Some(last_expr)
    }

    pub(super) fn parse_foreach_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwForeach));

        let kw = self.bump();
        let start = kw.span.start;
        let mut last_end = kw.span.end;

        self.expect(TokenKind::LParen, "expected '(' after 'foreach'");
        let expr = self.parse_expr().or_else(|| {
            self.error_here(
                ParseDiagnosticCode::ExpectedExpression,
                "expected expression after '('",
            );
            None
        });

        if self.at(TokenKind::KwAs) {
            let as_token = self.bump();
            last_end = as_token.span.end;
        } else {
            self.error_here(
                ParseDiagnosticCode::ExpectedToken,
                "expected 'as' in foreach",
            );
        }

        let key_or_val = self.parse_expr();
        let (key, value) = if self.eat(TokenKind::FatArrow) {
            let v = self.parse_expr();
            (key_or_val, v)
        } else {
            (None, key_or_val)
        };

        let rp =
            self.expect(TokenKind::RParen, "expected ')' after foreach header");
        if let Some(token) = rp.as_ref() {
            last_end = token.span.end;
        }

        if self.eat(TokenKind::Colon) {
            let (body, end) = self.parse_until_any(&[TokenKind::KwEndForeach]);
            last_end = end;
            let end_token =
                self.expect(TokenKind::KwEndForeach, "expected 'endforeach'");
            if let Some(token) = end_token {
                last_end = token.span.end;
            }
            let _ = self.eat(TokenKind::Semicolon);
            let span = Span {
                start,
                end: last_end,
            };
            return Some(Stmt::Foreach {
                expr,
                key,
                value,
                body,
                span,
            });
        }

        let body = self.parse_stmt_block_or_single(start, &mut last_end);

        let span = Span {
            start,
            end: last_end,
        };

        Some(Stmt::Foreach {
            expr,
            key,
            value,
            body,
            span,
        })
    }

    pub(super) fn parse_while_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwWhile));

        let kw = self.bump();
        let start = kw.span.start;
        let mut last_end = kw.span.end;

        self.expect(TokenKind::LParen, "expected '(' after 'while'");
        let cond = self.parse_expr();
        let rp = self
            .expect(TokenKind::RParen, "expected ')' after while condition");
        if let Some(token) = rp.as_ref() {
            last_end = token.span.end;
        }

        if self.eat(TokenKind::Colon) {
            let (body, end) = self.parse_until_any(&[TokenKind::KwEndWhile]);
            last_end = end;
            let end_token =
                self.expect(TokenKind::KwEndWhile, "expected 'endwhile'");
            if let Some(token) = end_token {
                last_end = token.span.end;
            }
            let _ = self.eat(TokenKind::Semicolon);
            let span = Span {
                start,
                end: last_end,
            };
            return Some(Stmt::While { cond, body, span });
        }

        let body = self.parse_stmt_block_or_single(start, &mut last_end);

        let span = Span {
            start,
            end: last_end,
        };

        Some(Stmt::While { cond, body, span })
    }

    pub(super) fn parse_do_while_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwDo));

        let do_token = self.bump();
        let start = do_token.span.start;
        let mut last_end = do_token.span.end;

        let body_block: Block = if self.eat(TokenKind::LBrace) {
            let lb = self.prev_span().unwrap();
            match self.parse_block_after_lbrace(lb.start) {
                Some((block, end)) => {
                    last_end = end;
                    block
                },
                None => {
                    let span = Span {
                        start: lb.start,
                        end: lb.start,
                    };
                    Block {
                        items: Vec::new(),
                        span,
                    }
                },
            }
        } else {
            let before = self.pos;
            if let Some(stmt) = self.parse_stmt() {
                last_end = stmt.span().end;
                let span = Span {
                    start: stmt.span().start,
                    end: stmt.span().end,
                };
                Block {
                    items: vec![stmt],
                    span,
                }
            } else {
                self.pos = before;
                self.error_here(
                    ParseDiagnosticCode::ExpectedStatement,
                    "expected statement after 'do'",
                );
                let span = Span {
                    start: last_end,
                    end: last_end,
                };
                Block {
                    items: Vec::new(),
                    span,
                }
            }
        };

        if !self.at(TokenKind::KwWhile) {
            self.error_here(
                ParseDiagnosticCode::ExpectedToken,
                "expected 'while' after do-statement body",
            );
        } else {
            let w = self.bump();
            last_end = w.span.end;
        }

        if let Some(lp) =
            self.expect(TokenKind::LParen, "expected '(' after 'while'")
        {
            last_end = lp.span.end;
        }

        let cond = if let Some(expr) = self.parse_expr() {
            last_end = expr.span().end;
            expr
        } else {
            self.error_here(
                ParseDiagnosticCode::ExpectedExpression,
                "expected condition expression in do-while",
            );
            Expr::Error {
                span: Span {
                    start: last_end,
                    end: last_end,
                },
            }
        };

        if let Some(rp) =
            self.expect(TokenKind::RParen, "expected ')' to close condition")
        {
            last_end = rp.span.end;
        }

        let semi_span = if let Some(semi) =
            self.expect(TokenKind::Semicolon, "expected ';' after do-while")
        {
            semi.span
        } else {
            self.recover_after_stmt_like(last_end)
        };
        last_end = semi_span.end;

        Some(Stmt::DoWhile {
            cond,
            body: body_block,
            span: Span {
                start,
                end: last_end,
            },
        })
    }

    #[inline]
    pub(crate) fn recover_after_stmt_like(&mut self, last_end: u32) -> Span {
        self.recover_to_any(&[
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Dollar,
            TokenKind::Ident,
            TokenKind::AttrOpen,
        ]);
        self.prev_span().unwrap_or(Span {
            start: last_end,
            end: last_end,
        })
    }

    fn parse_stmt_block_or_single(
        &mut self,
        start: u32,
        last_end: &mut u32,
    ) -> Block {
        if self.eat(TokenKind::LBrace) {
            let lb = self.prev_span().unwrap();
            if let Some((block, end)) = self.parse_block_after_lbrace(lb.start)
            {
                *last_end = end;
                return block;
            }
        }
        let stmt = self.parse_stmt().unwrap_or(Stmt::Noop {
            span: self.prev_span().unwrap_or(Span {
                start,
                end: *last_end,
            }),
        });
        *last_end = stmt.span().end;

        Block {
            items: vec![stmt],
            span: Span {
                start,
                end: *last_end,
            },
        }
    }
}
