use crate::ast::{Block, Stmt};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

impl<'src> Parser<'src> {
    pub(super) fn parse_namespace_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwNamespace));

        let ns_token = self.bump();
        let ns_start = ns_token.span.start;

        let ns_name_opt = if self.at(TokenKind::Backslash)
            || matches!(self.peek().map(|t| t.kind), Some(TokenKind::Ident))
        {
            self.parse_qualified_name()
        } else {
            None
        };

        let err_pos = ns_name_opt
            .as_ref()
            .map(|n| n.span.end)
            .unwrap_or(ns_token.span.end);

        if self.at(TokenKind::Semicolon) {
            self.bump();
            let (body_block, body_end) = self.parse_namespace_body_until_next();
            let span = Span {
                start: ns_start,
                end: body_end,
            };
            return Some(Stmt::Namespace {
                name: ns_name_opt,
                body: body_block,
                span,
            });
        }

        if self.at(TokenKind::LBrace) {
            let lb = self.bump();
            let (body_block, body_end) =
                match self.parse_block_after_lbrace(lb.span.start) {
                    Some((block, end)) => (block, end),
                    None => {
                        let span = Span::at(lb.span.start);
                        (
                            Block {
                                items: Vec::new(),
                                span,
                            },
                            span.end,
                        )
                    },
                };
            let span = Span {
                start: ns_start,
                end: body_end,
            };
            return Some(Stmt::Namespace {
                name: ns_name_opt,
                body: body_block,
                span,
            });
        }

        self.error_and_recover(
            Diagnostic::error_from_code(
                ParseDiagnosticCode::expected_tokens([
                    TokenKind::Semicolon,
                    TokenKind::LBrace,
                ]),
                Span::at(err_pos),
            ),
            &[
                TokenKind::LBrace,
                TokenKind::Semicolon,
                TokenKind::KwNamespace,
                TokenKind::RBrace,
            ],
        );

        let last_end =
            ns_name_opt.as_ref().map(|n| n.span.end).unwrap_or(ns_start);
        let empty = Block {
            items: Vec::new(),
            span: Span::at(last_end),
        };
        let span = Span {
            start: ns_start,
            end: last_end,
        };
        Some(Stmt::Namespace {
            name: ns_name_opt,
            body: empty,
            span,
        })
    }

    fn parse_namespace_body_until_next(&mut self) -> (Block, u32) {
        let start = self.current_span().start;
        let mut items = Vec::new();

        while !self.at(TokenKind::Eof) && !self.at(TokenKind::KwNamespace) {
            if let Some(stmt) = self.parse_stmt() {
                items.push(stmt);
            } else {
                if !self.at(TokenKind::Eof) {
                    let _ = self.bump();
                }
            }
        }

        let end = items.last().map(|s| s.span().end).unwrap_or(start);

        (
            Block {
                items,
                span: Span { start, end },
            },
            end,
        )
    }
}
