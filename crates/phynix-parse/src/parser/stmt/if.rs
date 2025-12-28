use crate::ast::{Block, Expr, Stmt};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

impl<'src> Parser<'src> {
    pub(super) fn parse_if_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwIf));

        let if_token = self.bump();
        let start_pos = if_token.span.start;
        let mut last_end = if_token.span.end;

        self.expect_or_err(TokenKind::LParen, &mut last_end);

        let cond_expr = self.parse_expr_or_err(&mut last_end);

        let have_rparen = self.expect_or_err(TokenKind::RParen, &mut last_end);

        if self.eat(TokenKind::Colon) {
            let (then_block, mut last_end2) = self.parse_until_any(&[
                TokenKind::KwElseIf,
                TokenKind::KwElse,
                TokenKind::KwEndIf,
            ]);

            let mut else_if_blocks: Vec<(Expr, Block)> = Vec::new();

            while self.at(TokenKind::KwElseIf) {
                let elseif_kw = self.bump();
                let mut elseif_end = elseif_kw.span.end;
                self.expect_or_err(TokenKind::LParen, &mut elseif_end);
                let cond = self.parse_expr().unwrap_or(Expr::Error {
                    span: elseif_kw.span,
                });
                elseif_end = cond.span().end;
                self.expect_or_err(TokenKind::RParen, &mut elseif_end);
                self.expect_or_err(TokenKind::Colon, &mut elseif_end);

                let (block, end) = self.parse_until_any(&[
                    TokenKind::KwElseIf,
                    TokenKind::KwElse,
                    TokenKind::KwEndIf,
                ]);
                last_end2 = end;
                else_if_blocks.push((cond, block));
            }

            let mut else_block_opt: Option<Block> = None;
            if self.at(TokenKind::KwElse) {
                let else_kw = self.bump();
                let mut else_end = else_kw.span.end;
                self.expect_or_err(TokenKind::Colon, &mut else_end);
                let (block, end) = self.parse_until_any(&[TokenKind::KwEndIf]);
                last_end2 = end;
                else_block_opt = Some(block);
            }

            self.expect_or_err(TokenKind::KwEndIf, &mut last_end2);
            let _ = self.eat(TokenKind::Semicolon);

            let full_span = Span {
                start: start_pos,
                end: last_end2,
            };
            return Some(Stmt::If {
                cond: cond_expr,
                then_block,
                else_if_blocks,
                else_block: else_block_opt,
                span: full_span,
            });
        }

        let then_block = if have_rparen && self.eat(TokenKind::Colon) {
            let (block, end) = self.parse_until_any(&[
                TokenKind::KwElseIf,
                TokenKind::KwElse,
                TokenKind::KwEndIf,
            ]);
            last_end = end;
            block
        } else if have_rparen && self.at(TokenKind::LBrace) {
            self.parse_branch_block(&mut last_end)
        } else if have_rparen {
            // Single-statement if-body: if (cond) stmt;
            if let Some(stmt) = self.parse_stmt() {
                let span = Span {
                    start: stmt.span().start,
                    end: stmt.span().end,
                };
                last_end = span.end;
                Block {
                    items: vec![stmt],
                    span,
                }
            } else {
                let fake = Span::at(last_end);
                Block {
                    items: Vec::new(),
                    span: fake,
                }
            }
        } else {
            Block {
                items: Vec::new(),
                span: Span::at(last_end),
            }
        };

        let mut else_if_blocks: Vec<(Expr, Block)> = Vec::new();
        let mut else_block_opt: Option<Block> = None;

        loop {
            if self.at(TokenKind::KwElseIf) {
                let elseif_kw = self.bump();
                let mut ei_end = elseif_kw.span.end;
                self.expect_or_err(TokenKind::LParen, &mut ei_end);
                let cond = self.parse_expr_or_err(&mut ei_end);
                self.expect_or_err(TokenKind::RParen, &mut ei_end);
                if !self.eat(TokenKind::Colon) {
                    // Non-colon form:
                    // - elseif (...) { ... }
                    // - elseif (...) stmt;
                    let block = if self.at(TokenKind::LBrace) {
                        self.parse_branch_block(&mut last_end)
                    } else {
                        // Single-statement elseif-body: elseif (cond) stmt;
                        if let Some(stmt) = self.parse_stmt() {
                            let span = Span {
                                start: stmt.span().start,
                                end: stmt.span().end,
                            };
                            last_end = span.end;
                            Block {
                                items: vec![stmt],
                                span,
                            }
                        } else {
                            self.error(Diagnostic::error_from_code(
                                ParseDiagnosticCode::ExpectedStatement,
                                Span::at(last_end),
                            ));
                            Block {
                                items: Vec::new(),
                                span: Span::at(last_end),
                            }
                        }
                    };

                    else_if_blocks.push((cond, block));
                    continue;
                }
                let (block, end) = self.parse_until_any(&[
                    TokenKind::KwElseIf,
                    TokenKind::KwElse,
                    TokenKind::KwEndIf,
                ]);
                last_end = end;
                else_if_blocks.push((cond, block));
                continue;
            }
            // ... (skipping else)

            if self.at(TokenKind::KwElse) {
                let _else = self.bump();

                if self.eat(TokenKind::Colon) {
                    let (block, end) =
                        self.parse_until_any(&[TokenKind::KwEndIf]);
                    last_end = end;
                    else_block_opt = Some(block);
                } else if self.at(TokenKind::KwIf) {
                    let block_start = _else.span.start;
                    let mut block_end = _else.span.end;
                    let mut items = Vec::new();

                    if let Some(if_stmt) = self.parse_if_stmt() {
                        block_end = if_stmt.span().end;
                        items.push(if_stmt);
                    } else {
                        self.error(Diagnostic::error_from_code(
                            ParseDiagnosticCode::expected_token(
                                TokenKind::KwIf,
                            ),
                            Span::at(block_end),
                        ));
                    }

                    let block = Block {
                        items,
                        span: Span {
                            start: block_start,
                            end: block_end,
                        },
                    };

                    last_end = block_end;
                    else_block_opt = Some(block);
                } else if self.at(TokenKind::LBrace) {
                    let block = self.parse_branch_block(&mut last_end);
                    else_block_opt = Some(block);
                } else {
                    // Single-statement else-body: else stmt;
                    if let Some(stmt) = self.parse_stmt() {
                        let span = Span {
                            start: stmt.span().start,
                            end: stmt.span().end,
                        };
                        last_end = span.end;
                        else_block_opt = Some(Block {
                            items: vec![stmt],
                            span,
                        });
                    } else {
                        self.error(Diagnostic::error_from_code(
                            ParseDiagnosticCode::ExpectedStatement,
                            Span::at(last_end),
                        ));
                    }
                }

                break;
            }

            break;
        }

        let span = Span {
            start: start_pos,
            end: last_end,
        };
        Some(Stmt::If {
            cond: cond_expr,
            then_block,
            else_if_blocks,
            else_block: else_block_opt,
            span,
        })
    }

    fn parse_branch_block(&mut self, last_end: &mut u32) -> Block {
        if let Some(token) = self.expect(TokenKind::LBrace) {
            let lb_start = token.span.start;
            return match self.parse_block_after_lbrace(lb_start) {
                Some((block, block_end)) => {
                    *last_end = block_end;
                    block
                },
                None => {
                    let fake_span = Span::at(lb_start);
                    *last_end = token.span.end;
                    Block {
                        items: Vec::new(),
                        span: fake_span,
                    }
                },
            };
        }

        self.error(Diagnostic::error_from_code(
            ParseDiagnosticCode::expected_token(TokenKind::LBrace),
            Span::at(*last_end),
        ));

        Block {
            items: Vec::new(),
            span: Span::at(*last_end),
        }
    }
}
