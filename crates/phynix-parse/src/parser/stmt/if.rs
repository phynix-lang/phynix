use crate::ast::{Block, Expr, Stmt};
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_if_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwIf));

        let if_token = self.bump();
        let start_pos = if_token.span.start;
        let mut last_end = if_token.span.end;

        let lp_token =
            self.expect(TokenKind::LParen, "expected '(' after 'if'");
        if let Some(lp) = lp_token.as_ref() {
            last_end = lp.span.end;
        }

        let cond_expr_opt = self.parse_expr();
        if let Some(cond_expr) = cond_expr_opt.as_ref() {
            last_end = cond_expr.span().end;
        } else {
            self.error_here("expected condition expression after '('");
        }

        let rparen_token =
            self.expect(TokenKind::RParen, "expected ')' after if condition");
        let have_rparen = rparen_token.is_some();
        if let Some(rp) = rparen_token.as_ref() {
            last_end = rp.span.end;
        }

        let parse_branch_block = |this: &mut Parser<'src>,
                                  last_end_ref: &mut u32,
                                  err_msg: &'static str|
         -> Block {
            let lbrace_token = this.expect(TokenKind::LBrace, err_msg);

            if let Some(lb) = lbrace_token.as_ref() {
                return match this.parse_block_after_lbrace(lb.span.start) {
                    Some((block, block_end)) => {
                        *last_end_ref = block_end;
                        block
                    },
                    None => {
                        let fake_span = Span {
                            start: lb.span.start,
                            end: lb.span.start,
                        };
                        *last_end_ref = fake_span.end;
                        Block {
                            items: Vec::new(),
                            span: fake_span,
                        }
                    },
                };
            }

            this.error_here(err_msg);
            let fake_span = Span {
                start: *last_end_ref,
                end: *last_end_ref,
            };
            Block {
                items: Vec::new(),
                span: fake_span,
            }
        };

        if self.eat(TokenKind::Colon) {
            let (then_block, mut last_end2) = self.parse_until_any(&[
                TokenKind::KwElseIf,
                TokenKind::KwElse,
                TokenKind::KwEndIf,
            ]);

            let mut else_if_blocks: Vec<(Expr, Block)> = Vec::new();

            while self.at(TokenKind::KwElseIf) {
                let _elseif = self.bump();
                let _ = self
                    .expect(TokenKind::LParen, "expected '(' after 'elseif'");
                let cond = self
                    .parse_expr()
                    .unwrap_or(Expr::Error { span: _elseif.span });
                let _ = self.expect(
                    TokenKind::RParen,
                    "expected ')' after elseif condition",
                );
                let _ = self
                    .expect(TokenKind::Colon, "expected ':' after elseif(...)");

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
                let _else = self.bump();
                let _ =
                    self.expect(TokenKind::Colon, "expected ':' after else");
                let (block, end) = self.parse_until_any(&[TokenKind::KwEndIf]);
                last_end2 = end;
                else_block_opt = Some(block);
            }

            let end_token = self.expect(
                TokenKind::KwEndIf,
                "expected 'endif' after if-colon block",
            );
            if let Some(token) = end_token {
                last_end2 = token.span.end;
            }
            let _ = self.eat(TokenKind::Semicolon);

            let full_span = Span {
                start: start_pos,
                end: last_end2,
            };
            return Some(Stmt::If {
                cond: cond_expr_opt.unwrap_or(Expr::Error {
                    span: if_token.span,
                }),
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
            parse_branch_block(
                self,
                &mut last_end,
                "expected '{' after if(...)",
            )
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
                let fake = Span {
                    start: last_end,
                    end: last_end,
                };
                Block {
                    items: Vec::new(),
                    span: fake,
                }
            }
        } else {
            Block {
                items: Vec::new(),
                span: Span {
                    start: last_end,
                    end: last_end,
                },
            }
        };

        let mut else_if_blocks: Vec<(Expr, Block)> = Vec::new();
        let mut else_block_opt: Option<Block> = None;

        loop {
            if self.at(TokenKind::KwElseIf) {
                let _elseif = self.bump();
                if let Some(_lp) = self
                    .expect(TokenKind::LParen, "expected '(' after 'elseif'")
                {
                }
                let cond_opt = self.parse_expr();
                let _ = self.expect(
                    TokenKind::RParen,
                    "expected ')' after elseif condition",
                );
                if !self.eat(TokenKind::Colon) {
                    let block = parse_branch_block(
                        self,
                        &mut last_end,
                        "expected '{' after elseif(...)",
                    );
                    if let Some(c) = cond_opt {
                        else_if_blocks.push((c, block));
                    }
                    continue;
                }
                let (block, end) = self.parse_until_any(&[
                    TokenKind::KwElseIf,
                    TokenKind::KwElse,
                    TokenKind::KwEndIf,
                ]);
                last_end = end;
                if let Some(c) = cond_opt {
                    else_if_blocks.push((c, block));
                }
                continue;
            }

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
                        self.error_here("expected 'if' after 'else'");
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
                } else {
                    let block = parse_branch_block(
                        self,
                        &mut last_end,
                        "expected '{' after 'else'",
                    );
                    else_block_opt = Some(block);
                }

                break;
            }

            break;
        }

        if self.at(TokenKind::KwEndIf) {
            last_end = self.bump().span.end;
            let _ = self.eat(TokenKind::Semicolon);
        }

        let full_span = Span {
            start: start_pos,
            end: last_end,
        };
        Some(Stmt::If {
            cond: cond_expr_opt.unwrap_or(Expr::Error {
                span: if_token.span,
            }),
            then_block,
            else_if_blocks,
            else_block: else_block_opt,
            span: full_span,
        })
    }
}
