use crate::ast::{ArrayItemExpr, Expr, StringStyle};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::{Token, TokenKind};
use phynix_core::{Span, Spanned};

impl<'src> Parser<'src> {
    pub(crate) fn parse_int_literal(&mut self) -> Option<Expr> {
        debug_assert!(self.at(TokenKind::Int));

        let token = self.bump();
        let string = self.slice_without_underscores(&token);
        let t = string.as_str();

        let (radix, digits) = if let Some(rest) =
            t.strip_prefix("0x").or(t.strip_prefix("0X"))
        {
            (16, rest)
        } else if let Some(rest) = t.strip_prefix("0b").or(t.strip_prefix("0B"))
        {
            (2, rest)
        } else if let Some(rest) = t.strip_prefix("0o").or(t.strip_prefix("0O"))
        {
            (8, rest)
        } else {
            (10, t)
        };

        match i64::from_str_radix(digits, radix) {
            Ok(v) => Some(Expr::IntLiteral {
                value: v,
                span: token.span,
            }),
            Err(_) => {
                self.error(Diagnostic::error_from_code(
                    ParseDiagnosticCode::InvalidIntLiteral,
                    token.span,
                ));
                Some(Expr::IntLiteral {
                    value: 0,
                    span: token.span,
                })
            },
        }
    }

    pub(crate) fn parse_float_literal(&mut self) -> Option<Expr> {
        debug_assert!(self.at(TokenKind::Float));

        let token = self.bump();
        let string = self.slice_without_underscores(&token);
        let t = string.as_str();

        match t.parse::<f64>() {
            Ok(v) => Some(Expr::FloatLiteral {
                value: v,
                span: token.span,
            }),
            Err(_) => {
                self.error(Diagnostic::error_from_code(
                    ParseDiagnosticCode::InvalidFloatLiteral,
                    token.span,
                ));
                Some(Expr::FloatLiteral {
                    value: 0.0,
                    span: token.span,
                })
            },
        }
    }

    pub(super) fn parse_string_like_literal(&mut self) -> Option<Expr> {
        debug_assert!(self.at_any(&[
            TokenKind::StrSq,
            TokenKind::StrDq,
            TokenKind::StrBt
        ]));

        let token = self.bump();
        match token.kind {
            TokenKind::StrSq => Some(Expr::StringLiteral {
                style: StringStyle::SingleQuoted,
                span: token.span,
            }),
            TokenKind::StrDq => {
                let text = self.slice(&token);

                if text.starts_with("<<<") {
                    let bytes = text.as_bytes();
                    let style = if bytes.get(3) == Some(&b'\'') {
                        StringStyle::Nowdoc
                    } else {
                        StringStyle::Heredoc
                    };

                    Some(Expr::StringLiteral {
                        style,
                        span: token.span,
                    })
                } else {
                    Some(Expr::StringLiteral {
                        style: StringStyle::DoubleQuoted,
                        span: token.span,
                    })
                }
            },
            TokenKind::StrBt => Some(Expr::ShellExec { span: token.span }),
            _ => None,
        }
    }

    pub(super) fn parse_array_literal(
        &mut self,
        already_saw_lbracket: bool,
        lbracket_span: Option<Span>,
    ) -> Option<Expr> {
        let array_start_span = if already_saw_lbracket {
            lbracket_span.unwrap_or(self.current_span())
        } else {
            let lb_span = self.current_span();
            let mut lb_end = lb_span.start;
            if !self.expect_or_err(TokenKind::LBracket, &mut lb_end) {
                return None;
            }
            lb_span
        };
        let start_pos = array_start_span.start;

        let (items, end_pos) =
            self.parse_array_items_until(TokenKind::RBracket, array_start_span);

        Some(Expr::ArrayLiteral {
            items,
            span: Span {
                start: start_pos,
                end: end_pos,
            },
        })
    }

    pub(crate) fn parse_array_construct(&mut self) -> Option<Expr> {
        debug_assert!(self.at(TokenKind::KwArray));

        let array_token = self.bump();
        let mut lp_end = array_token.span.end;
        if !self.expect_or_err(TokenKind::LParen, &mut lp_end) {
            return None;
        }
        let lp_span = Span {
            start: array_token.span.end,
            end: lp_end,
        };
        let start_pos = array_token.span.start;

        let (items, end_pos) =
            self.parse_array_items_until(TokenKind::RParen, lp_span);

        Some(Expr::ArrayLiteral {
            items,
            span: Span {
                start: start_pos,
                end: end_pos,
            },
        })
    }

    #[inline]
    fn parse_array_literal_expr_with_default(
        &mut self,
        default_span: Span,
    ) -> Expr {
        match self.parse_array_literal(true, Some(default_span)) {
            Some(expr) => expr,
            None => {
                let fake_span = Span::at(default_span.end);
                Expr::Error { span: fake_span }
            },
        }
    }

    #[inline]
    fn parse_array_construct_expr_with_default(
        &mut self,
        default_span: Span,
    ) -> Expr {
        match self.parse_array_construct() {
            Some(expr) => expr,
            None => {
                let fake_span = Span::at(default_span.end);
                Expr::Error { span: fake_span }
            },
        }
    }

    #[inline]
    fn eat_end_get_end(&mut self, end: TokenKind) -> Option<u32> {
        let span = self.current_span();
        if self.eat(end) {
            Some(span.end)
        } else {
            None
        }
    }

    pub(crate) fn parse_isset_expr(&mut self) -> Option<Expr> {
        debug_assert!(self.at(TokenKind::KwIsset));

        let kw = self.bump();
        let start = kw.span.start;

        let mut lp_end = kw.span.end;
        self.expect_or_err(TokenKind::LParen, &mut lp_end);

        let mut exprs: Vec<Expr> = Vec::new();

        let rp_span = self.current_span();
        if self.eat(TokenKind::RParen) {
            let end = rp_span.end;
            self.error(Diagnostic::error_from_code(
                ParseDiagnosticCode::ExpectedIdent,
                Span::at(lp_end),
            ));
            return Some(Expr::Isset {
                exprs,
                span: Span { start, end },
            });
        }

        loop {
            let loop_start = self.current_span().start;
            if let Some(e) = self.parse_expr() {
                exprs.push(e);
            } else {
                self.error_and_recover(
                    Diagnostic::error_from_code(
                        ParseDiagnosticCode::ExpectedExpression,
                        Span::at(loop_start),
                    ),
                    &[TokenKind::Comma, TokenKind::RParen],
                );
            }

            if self.eat(TokenKind::Comma) {
                if self.at(TokenKind::RParen) {
                    break;
                }
                continue;
            }

            break;
        }

        let mut end = kw.span.end;
        self.expect_or_err(TokenKind::RParen, &mut end);

        Some(Expr::Isset {
            exprs,
            span: Span { start, end },
        })
    }

    pub(crate) fn parse_empty_expr(&mut self) -> Option<Expr> {
        debug_assert!(self.at(TokenKind::KwEmpty));

        let kw = self.bump();
        let start = kw.span.start;

        let lp_span = self.current_span();
        let lp_end = if self.expect(TokenKind::LParen).is_some() {
            self.current_span().start
        } else {
            self.error(Diagnostic::error_from_code(
                ParseDiagnosticCode::expected_token(TokenKind::LParen),
                Span::at(kw.span.end),
            ));
            kw.span.end
        };

        let expr = match self.parse_expr() {
            Some(e) => e,
            None => {
                self.error_and_recover(
                    Diagnostic::error_from_code(
                        ParseDiagnosticCode::ExpectedExpression,
                        Span::at(lp_end),
                    ),
                    &[TokenKind::RParen],
                );
                Expr::Error {
                    span: Span::at(lp_span.end),
                }
            },
        };

        let mut end = expr.span().end;
        self.expect_or_err(TokenKind::RParen, &mut end);

        Some(Expr::Empty {
            expr: Box::new(expr),
            span: Span { start, end },
        })
    }

    #[inline]
    fn slice_without_underscores(&self, token: &Token) -> String {
        let s = self.slice(token);
        let mut out = String::with_capacity(s.len());
        for &b in s.as_bytes() {
            if b != b'_' {
                out.push(b as char);
            }
        }
        out
    }

    fn parse_array_items_until(
        &mut self,
        close: TokenKind,
        default_span: Span,
    ) -> (Vec<ArrayItemExpr>, u32) {
        let mut items: Vec<ArrayItemExpr> = Vec::new();

        if let Some(end) = self.eat_end_get_end(close) {
            return (items, end);
        }

        loop {
            let mut key_expr: Option<Expr> = None;
            let mut is_unpack = false;

            let item_start: u32;
            let first_expr: Expr;

            if self.at(TokenKind::Ellipsis) {
                let ell = self.bump();
                is_unpack = true;
                item_start = ell.span.start;

                first_expr = self.parse_expr().unwrap_or_else(|| {
                    self.error_and_recover(
                        Diagnostic::error_from_code(
                            ParseDiagnosticCode::ExpectedExpression,
                            Span::at(ell.span.end),
                        ),
                        &[TokenKind::Comma, close],
                    );
                    Expr::Error {
                        span: Span::at(ell.span.end),
                    }
                });
            } else if self.at(TokenKind::LBracket) {
                let lb = self.bump();
                item_start = lb.span.start;
                first_expr =
                    self.parse_array_literal_expr_with_default(default_span);
            } else if self.at(TokenKind::KwArray) {
                first_expr =
                    self.parse_array_construct_expr_with_default(default_span);
                item_start = first_expr.span().start;
            } else if let Some(first) = self.parse_expr() {
                item_start = first.span().start;
                first_expr = first;
            } else {
                let err_pos = self.current_span().start;
                self.error_and_recover(
                    Diagnostic::error_from_code(
                        ParseDiagnosticCode::ExpectedExpression, // TODO: Better code?
                        Span::at(err_pos),
                    ),
                    &[TokenKind::Comma, close],
                );
                let fake = Span::at(default_span.end);
                item_start = fake.start;
                first_expr = Expr::Error { span: fake };
            }

            let value_expr: Expr;

            let fat_arrow_span = self.current_span();
            if !is_unpack && self.eat(TokenKind::FatArrow) {
                key_expr = Some(first_expr);

                if self.at(TokenKind::LBracket) {
                    let lb = self.bump();
                    value_expr =
                        self.parse_array_literal_expr_with_default(lb.span);
                } else if self.at(TokenKind::KwArray) {
                    value_expr = self
                        .parse_array_construct_expr_with_default(default_span);
                } else {
                    let mut fat_arrow_end = fat_arrow_span.end;
                    value_expr = self.parse_expr_or_err(&mut fat_arrow_end);
                }
            } else {
                value_expr = first_expr;
                if is_unpack && self.at(TokenKind::FatArrow) {
                    let arrow_span = self.current_span();
                    self.error(Diagnostic::error_from_code(
                        ParseDiagnosticCode::UnpackedArrayItemWithFatArrow,
                        arrow_span,
                    ));
                }
            }

            let item_end = value_expr.span().end;
            items.push(ArrayItemExpr {
                key: key_expr,
                value: value_expr,
                unpack: is_unpack,
                span: Span {
                    start: item_start,
                    end: item_end,
                },
            });

            let had_comma = self.eat(TokenKind::Comma);

            let close_span = self.current_span();
            if self.eat(close) {
                let end_pos = close_span.end;
                return (items, end_pos);
            }

            if had_comma {
                continue;
            }

            self.error_and_recover(
                Diagnostic::error_from_code(
                    ParseDiagnosticCode::expected_tokens([
                        TokenKind::Comma,
                        TokenKind::RBracket,
                    ]),
                    Span::at(item_end),
                ),
                &[
                    close,
                    TokenKind::Semicolon,
                    TokenKind::Comma,
                    TokenKind::RBrace,
                ],
            );

            let end_pos = self.eat_end_get_end(close).unwrap_or(item_end);
            return (items, end_pos);
        }
    }
}
