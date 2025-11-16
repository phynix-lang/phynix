use crate::ast::Stmt;
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_declare_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwDeclare));

        let declare_token = self.bump();
        let start_pos = declare_token.span.start;
        let mut last_end = declare_token.span.end;

        let lparen_token =
            self.expect(TokenKind::LParen, "expected '(' after declare");
        if let Some(lp) = lparen_token.as_ref() {
            last_end = lp.span.end;
        } else {
            return Some(Stmt::Declare {
                strict_types: None,
                span: Span {
                    start: start_pos,
                    end: last_end,
                },
            });
        }

        let (dir_name_text, _dir_name_span) = match self.peek() {
            Some(token) if matches!(token.kind, TokenKind::Ident) => {
                let name_token = self.bump();
                let span = name_token.span;
                last_end = span.end;
                let text = self.slice(&name_token);
                (Some(text), span)
            },
            _ => {
                self.error_here("expected directive name in declare(...)");
                (None, lparen_token.unwrap().span)
            },
        };

        let saw_eq = self.eat(TokenKind::Eq);
        if !saw_eq {
            self.error_here("expected '=' after directive name");
        }

        let mut int_text_opt: Option<&str> = None;
        if let Some(token) = self.peek() {
            if matches!(token.kind, TokenKind::Int) {
                let int_token = self.bump();
                last_end = int_token.span.end;
                int_text_opt = Some(self.slice(&int_token));
            } else {
                if saw_eq {
                    self.error_here("expected integer literal after '='");
                }
            }
        }

        let strict_types_val = if let (Some("strict_types"), Some(int_txt)) =
            (dir_name_text, int_text_opt)
        {
            Some(int_txt == "1")
        } else {
            None
        };

        let rparen_token =
            self.expect(TokenKind::RParen, "expected ')' after declare(...)");
        if let Some(rp) = rparen_token.as_ref() {
            last_end = rp.span.end;
        }

        if self.eat(TokenKind::LBrace) {
            let mut depth = 1;
            let mut last_stmt_end: u32;

            while depth > 0 && !self.at(TokenKind::Eof) {
                if self.eat(TokenKind::LBrace) {
                    depth += 1;
                    continue;
                }
                if let Some(token) = self.peek() {
                    if matches!(token.kind, TokenKind::RBrace) {
                        let rb = self.bump();
                        last_end = rb.span.end;
                        depth -= 1;
                        continue;
                    }
                }

                if let Some(stmt) = self.parse_stmt() {
                    last_stmt_end = stmt.span().end;
                    last_end = last_stmt_end;
                } else {
                    let _ = self.bump();
                }
            }
        } else {
            let semi_token = self.expect(
                TokenKind::Semicolon,
                "expected ';' after declare(...)",
            );
            if let Some(semi) = semi_token.as_ref() {
                last_end = semi.span.end;
            } else {
                self.recover_to_any(&[
                    TokenKind::Semicolon,
                    TokenKind::RBrace,
                    TokenKind::AttrOpen,
                    TokenKind::Dollar,
                    TokenKind::Ident,
                    TokenKind::KwFunction,
                    TokenKind::KwClass,
                ]);
            }
        }

        let full_span = Span {
            start: start_pos,
            end: last_end,
        };

        Some(Stmt::Declare {
            strict_types: strict_types_val,
            span: full_span,
        })
    }
}
