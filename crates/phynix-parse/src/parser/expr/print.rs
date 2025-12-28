use crate::ast::Expr;
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

impl<'src> Parser<'src> {
    pub(crate) fn parse_print_expr(&mut self) -> Option<Expr> {
        debug_assert!(self.at(TokenKind::KwPrint));

        let kw = self.bump();
        let start = kw.span.start;

        let inner = match self.parse_prefix_term() {
            Some(e) => e,
            None => {
                self.error_and_recover(
                    Diagnostic::error_from_code(
                        ParseDiagnosticCode::ExpectedExpression,
                        Span::at(kw.span.end),
                    ),
                    &[
                        TokenKind::Semicolon,
                        TokenKind::Comma,
                        TokenKind::RParen,
                        TokenKind::RBracket,
                        TokenKind::RBrace,
                        TokenKind::PhpClose,
                    ],
                );
                Expr::Error { span: kw.span }
            },
        };

        let end = inner.span().end;
        Some(Expr::Print {
            expr: Box::new(inner),
            span: Span { start, end },
        })
    }
}
