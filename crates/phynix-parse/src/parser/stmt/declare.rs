use crate::ast::Stmt;
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

impl<'src> Parser<'src> {
    pub(super) fn parse_declare_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwDeclare));

        let declare_token = self.bump();
        let start_pos = declare_token.span.start;
        let mut last_end = declare_token.span.end;

        if !self.expect_or_err(TokenKind::LParen, &mut last_end) {
            return Some(Stmt::Declare {
                strict_types: None,
                span: Span {
                    start: start_pos,
                    end: last_end,
                },
            });
        }

        let lparen_span = Span::at(last_end);
        let (dir_name_text, _dir_name_span) = match self.peek() {
            Some(token) if matches!(token.kind, TokenKind::Ident) => {
                let name_token = self.bump();
                let span = name_token.span;
                last_end = span.end;
                let text = self.slice(&name_token);
                (Some(text), span)
            },
            _ => {
                self.error(Diagnostic::error_from_code(
                    ParseDiagnosticCode::ExpectedIdent,
                    Span::at(last_end),
                ));
                (None, lparen_span)
            },
        };

        let saw_eq = self.expect_or_err(TokenKind::Eq, &mut last_end);

        let mut int_text_opt: Option<&str> = None;
        if let Some(token) = self.peek() {
            if matches!(token.kind, TokenKind::Int) {
                let int_token = self.bump();
                last_end = int_token.span.end;
                int_text_opt = Some(self.slice(&int_token));
            } else {
                if saw_eq {
                    self.error(Diagnostic::error_from_code(
                        ParseDiagnosticCode::ExpectedIntLiteral,
                        Span::at(last_end),
                    ));
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

        self.expect_or_err(TokenKind::RParen, &mut last_end);

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
            if !self.expect_or_err(TokenKind::Semicolon, &mut last_end) {
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
