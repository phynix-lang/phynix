use crate::ast::Stmt;
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::Span;

impl<'src> Parser<'src> {
    pub(super) fn parse_global_stmt(&mut self) -> Option<Stmt> {
        assert!(self.at(TokenKind::KwGlobal));

        let global_token = self.bump();
        let mut end = global_token.span.end;

        loop {
            if self.at(TokenKind::VarIdent) {
                end = self.bump().span.end;
            } else if self.eat(TokenKind::Dollar) {
                let dollar_end = self.current_span().start;
                if let Some(id) = self.expect_ident_or_err(&mut end) {
                    end = id.span.end;
                } else {
                    self.error_and_recover(
                        Diagnostic::error_from_code(
                            ParseDiagnosticCode::ExpectedIdent,
                            Span::at(dollar_end),
                        ),
                        &[TokenKind::Comma, TokenKind::Semicolon],
                    );
                }
            } else {
                self.error_and_recover(
                    Diagnostic::error_from_code(
                        ParseDiagnosticCode::ExpectedIdent,
                        Span::at(end),
                    ),
                    &[TokenKind::Semicolon],
                );
                break;
            }

            if self.eat(TokenKind::Comma) {
                continue;
            }
            break;
        }

        if !self.expect_or_err(TokenKind::Semicolon, &mut end) {
            self.recover_to_any(&[TokenKind::Semicolon, TokenKind::RBrace]);
        }

        Some(Stmt::Global {
            span: Span {
                start: global_token.span.start,
                end,
            },
        })
    }
}
