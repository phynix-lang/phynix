use crate::ast::{Expr, MatchArm, MatchPattern};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

impl<'src> Parser<'src> {
    pub(super) fn parse_match_expr(&mut self) -> Option<Expr> {
        debug_assert!(self.at(TokenKind::KwMatch));

        let match_token = self.bump();
        let start = match_token.span.start;
        let mut last_end = match_token.span.end;

        self.expect_or_err(TokenKind::LParen, &mut last_end);

        let scrutinee = self.parse_expr_or_err(&mut last_end);

        self.expect_or_err(TokenKind::RParen, &mut last_end);

        let lb_start = self.current_span().start;
        self.expect_or_err(TokenKind::LBrace, &mut last_end);

        let mut arms: Vec<MatchArm> = Vec::new();

        loop {
            if self.at(TokenKind::RBrace) {
                break;
            }

            let mut patterns: Vec<MatchPattern> = Vec::new();

            if self.at(TokenKind::KwDefault) {
                let d = self.bump();
                patterns.push(MatchPattern::Default { span: d.span });
                last_end = d.span.end;
            } else {
                let first_pat = match self.parse_expr() {
                    Some(expr) => {
                        last_end = expr.span().end;
                        MatchPattern::Expr(expr)
                    },
                    None => {
                        self.error_and_recover(
                            Diagnostic::error_from_code(
                                ParseDiagnosticCode::expected_one_of([
                                    ParseDiagnosticCode::ExpectedExpression,
                                    ParseDiagnosticCode::expected_token(
                                        TokenKind::KwDefault,
                                    ),
                                ]),
                                Span::at(last_end),
                            ),
                            &[
                                TokenKind::FatArrow,
                                TokenKind::Comma,
                                TokenKind::RBrace,
                            ],
                        );
                        MatchPattern::Expr(Expr::Error {
                            span: Span::at(last_end),
                        })
                    },
                };
                patterns.push(first_pat);

                while self.eat(TokenKind::Comma) {
                    patterns.push(MatchPattern::Expr(
                        self.parse_expr_or_err(&mut last_end),
                    ));
                }
            }

            self.expect_or_err(TokenKind::FatArrow, &mut last_end);

            let arm_expr = self.parse_expr_or_err(&mut last_end);

            let mut arm_end = arm_expr.span().end;
            self.eat_and_update_end(TokenKind::Comma, &mut arm_end);

            arms.push(MatchArm {
                patterns,
                expr: arm_expr,
                span: Span {
                    start,
                    end: arm_end,
                },
            });

            if self.at(TokenKind::RBrace) {
                break;
            }
        }

        let rb = self.expect_or_err(TokenKind::RBrace, &mut last_end);
        let end = if rb { last_end } else { lb_start.max(last_end) };

        Some(Expr::Match {
            scrutinee: Box::new(scrutinee),
            arms,
            span: Span { start, end },
        })
    }
}
