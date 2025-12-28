use crate::ast::{Block, Expr, Stmt};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

impl<'src> Parser<'src> {
    pub(super) fn parse_for_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwFor));

        let kw = self.bump();
        let start = kw.span.start;
        let mut last_end = kw.span.end;

        self.expect_or_err(TokenKind::LParen, &mut last_end);

        let init = if !self.at(TokenKind::Semicolon) {
            self.parse_for_expr_list(&[TokenKind::Semicolon, TokenKind::RParen])
        } else {
            None
        };
        self.expect_or_err(TokenKind::Semicolon, &mut last_end);

        let cond = if !self.at(TokenKind::Semicolon) {
            self.parse_for_expr_list(&[TokenKind::Semicolon, TokenKind::RParen])
        } else {
            None
        };
        self.expect_or_err(TokenKind::Semicolon, &mut last_end);

        let step = if !self.at(TokenKind::RParen) {
            self.parse_for_expr_list(&[TokenKind::RParen])
        } else {
            None
        };
        self.expect_or_err(TokenKind::RParen, &mut last_end);

        if self.eat(TokenKind::Colon) {
            let (body, end) = self.parse_until_any(&[TokenKind::KwEndFor]);
            last_end = end;

            self.expect_or_err(TokenKind::KwEndFor, &mut last_end);

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

        while let Some(comma) = self.expect(TokenKind::Comma) {
            let last_end = comma.span.end;
            if self.at_any(terminators) {
                self.error(Diagnostic::error_from_code(
                    ParseDiagnosticCode::ExpectedExpression,
                    Span::at(last_end),
                ));
                break;
            }

            match self.parse_expr() {
                Some(e) => {
                    last_expr = e;
                },
                None => {
                    self.error_and_recover(
                        Diagnostic::error_from_code(
                            ParseDiagnosticCode::ExpectedExpression,
                            Span::at(last_end),
                        ),
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

        self.expect_or_err(TokenKind::LParen, &mut last_end);

        let expr = if self.at(TokenKind::RParen) {
            None
        } else {
            Some(self.parse_expr_or_err(&mut last_end))
        };

        if self.at(TokenKind::KwAs) {
            let as_token = self.bump();
            last_end = as_token.span.end;
        } else {
            self.error(Diagnostic::error_from_code(
                ParseDiagnosticCode::expected_token(TokenKind::KwAs),
                Span::at(last_end),
            ));
        }

        let key_or_val = self.parse_expr();
        if let Some(ref e) = key_or_val {
            last_end = e.span().end;
        }
        let (key, value) = if let Some(arrow) = self.expect(TokenKind::FatArrow)
        {
            last_end = arrow.span.end;
            let v = self.parse_expr();
            if let Some(ref e) = v {
                last_end = e.span().end;
            }
            (key_or_val, v)
        } else {
            (None, key_or_val)
        };

        self.expect_or_err(TokenKind::RParen, &mut last_end);

        if self.eat(TokenKind::Colon) {
            let (body, end) = self.parse_until_any(&[TokenKind::KwEndForeach]);
            last_end = end;

            self.expect_or_err(TokenKind::KwEndForeach, &mut last_end);

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

        self.expect_or_err(TokenKind::LParen, &mut last_end);

        let cond = if self.at(TokenKind::RParen) {
            None
        } else {
            Some(self.parse_expr_or_err(&mut last_end))
        };

        self.expect_or_err(TokenKind::RParen, &mut last_end);

        if self.eat(TokenKind::Colon) {
            let (body, end) = self.parse_until_any(&[TokenKind::KwEndWhile]);
            last_end = end;

            self.expect_or_err(TokenKind::KwEndWhile, &mut last_end);

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

        let body_block: Block = if let Some(lb) = self.expect(TokenKind::LBrace)
        {
            let lb_start = lb.span.start;
            last_end = lb.span.end;
            match self.parse_block_after_lbrace(lb_start) {
                Some((block, end)) => {
                    last_end = end;
                    block
                },
                None => {
                    let span = Span::at(lb_start);
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
                self.error(Diagnostic::error_from_code(
                    ParseDiagnosticCode::ExpectedStatement,
                    Span::at(last_end),
                ));
                let span = Span::at(last_end);
                Block {
                    items: Vec::new(),
                    span,
                }
            }
        };

        self.expect_or_err(TokenKind::KwWhile, &mut last_end);
        self.expect_or_err(TokenKind::LParen, &mut last_end);

        let cond = self.parse_expr_or_err(&mut last_end);

        self.expect_or_err(TokenKind::RParen, &mut last_end);

        let semi_span = if let Some(semi) = self.expect(TokenKind::Semicolon) {
            semi.span
        } else {
            self.error(Diagnostic::error_from_code(
                ParseDiagnosticCode::expected_token(TokenKind::Semicolon),
                Span::at(last_end),
            ));
            Span::at(last_end)
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

    fn parse_stmt_block_or_single(
        &mut self,
        start: u32,
        last_end: &mut u32,
    ) -> Block {
        if let Some(lb) = self.expect(TokenKind::LBrace) {
            let lb_start = lb.span.start;
            *last_end = lb.span.end;
            if let Some((block, end)) = self.parse_block_after_lbrace(lb_start)
            {
                *last_end = end;
                return block;
            }
        }
        let stmt = self.parse_stmt().unwrap_or(Stmt::Noop {
            span: Span {
                start,
                end: *last_end,
            },
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
