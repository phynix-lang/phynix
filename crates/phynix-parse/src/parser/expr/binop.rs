use crate::ast::{BinOpKind, Expr, QualifiedName};
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_binop_prec_impl(
        &mut self,
        min_prec: u8,
    ) -> Option<Expr> {
        let mut left = self.parse_prefix_term()?;

        loop {
            if self.at(TokenKind::KwInstanceof) {
                let instanceof_token = self.bump();

                let class_qn = if self.at(TokenKind::Backslash)
                    || self.at(TokenKind::Ident)
                {
                    match self.parse_qualified_name(
                        "expected class after 'instanceof'",
                    ) {
                        Some(qn) => qn,
                        None => {
                            self.recover_to_any(&[
                                TokenKind::Semicolon,
                                TokenKind::Comma,
                                TokenKind::RParen,
                                TokenKind::RBracket,
                            ]);

                            let fake = self
                                .prev_span()
                                .unwrap_or(instanceof_token.span);
                            let span = Span {
                                start: left.span().start,
                                end: fake.end,
                            };

                            left = Expr::InstanceOf {
                                expr: Box::new(left),
                                class: QualifiedName {
                                    absolute: false,
                                    parts: vec![],
                                    span,
                                },
                                span,
                            };
                            continue;
                        },
                    }
                } else if self.at(TokenKind::VarIdent) {
                    let var_tok = self.bump();
                    let span = var_tok.span;
                    let ident = crate::ast::Ident { span };
                    QualifiedName {
                        absolute: false,
                        parts: vec![ident],
                        span,
                    }
                } else {
                    self.error_and_recover(
                        "expected class after 'instanceof'",
                        &[
                            TokenKind::Semicolon,
                            TokenKind::Comma,
                            TokenKind::RParen,
                            TokenKind::RBracket,
                        ],
                    );

                    let fake =
                        self.prev_span().unwrap_or(instanceof_token.span);
                    let span = Span {
                        start: left.span().start,
                        end: fake.end,
                    };

                    left = Expr::InstanceOf {
                        expr: Box::new(left),
                        class: QualifiedName {
                            absolute: false,
                            parts: vec![],
                            span,
                        },
                        span,
                    };
                    continue;
                };

                let span = Span {
                    start: left.span().start,
                    end: class_qn.span.end,
                };
                left = Expr::InstanceOf {
                    expr: Box::new(left),
                    class: class_qn,
                    span,
                };
                continue;
            }

            let Some(op_kind) = self.peek_binop() else {
                break;
            };
            let prec = precedence_of(op_kind);
            if prec < min_prec {
                break;
            }

            let op_token = self.bump();

            let mut right = match self.parse_prefix_term() {
                Some(expr) => expr,
                None => {
                    let err_span = op_token.span;
                    self.error_and_recover(
                        "expected expression after operator",
                        &[
                            TokenKind::Semicolon,
                            TokenKind::Comma,
                            TokenKind::RParen,
                            TokenKind::RBracket,
                            TokenKind::RBrace,
                        ],
                    );
                    let span = Span {
                        start: left.span().start,
                        end: err_span.end,
                    };
                    return Some(Expr::BinaryOp {
                        op: op_kind,
                        left: Box::new(left),
                        right: Box::new(Expr::Error { span: err_span }),
                        span,
                    });
                },
            };

            right = self.maybe_recurse_higher_prec(right, prec);

            let span = Span {
                start: left.span().start,
                end: right.span().end,
            };
            left = Expr::BinaryOp {
                op: op_kind,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }

        Some(left)
    }

    fn parse_binop_prec_from_left(
        &mut self,
        mut left: Expr,
        min_prec: u8,
    ) -> Expr {
        loop {
            let Some(op_kind) = self.peek_binop() else {
                break;
            };
            let prec = precedence_of(op_kind);
            if prec < min_prec {
                break;
            }

            let op_token = self.bump();

            let mut right = match self.parse_prefix_term() {
                Some(expr) => expr,
                None => {
                    let err_span = op_token.span;
                    self.error_and_recover(
                        "expected expression after operator",
                        &[
                            TokenKind::Semicolon,
                            TokenKind::Comma,
                            TokenKind::RParen,
                            TokenKind::RBracket,
                            TokenKind::RBrace,
                        ],
                    );
                    let span = Span {
                        start: left.span().start,
                        end: err_span.end,
                    };
                    return Expr::BinaryOp {
                        op: op_kind,
                        left: Box::new(left),
                        right: Box::new(Expr::Error { span: err_span }),
                        span,
                    };
                },
            };

            right = self.maybe_recurse_higher_prec(right, prec);

            let span = Span {
                start: left.span().start,
                end: right.span().end,
            };
            left = Expr::BinaryOp {
                op: op_kind,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }

        left
    }

    #[inline]
    fn maybe_recurse_higher_prec(&mut self, right: Expr, prec: u8) -> Expr {
        if self.higher_prec_follows(prec) {
            self.parse_binop_prec_from_left(right, prec + 1)
        } else {
            right
        }
    }

    #[inline]
    fn peek_binop(&self) -> Option<BinOpKind> {
        token_to_binop(self.nth_kind(0))
    }

    #[inline]
    fn higher_prec_follows(&self, cur_prec: u8) -> bool {
        self.peek_binop()
            .map(|k| precedence_of(k) > cur_prec)
            .unwrap_or(false)
    }
}

#[inline]
pub fn token_to_binop(kind: &TokenKind) -> Option<BinOpKind> {
    match kind {
        // Arithmetic
        TokenKind::Plus => Some(BinOpKind::Add),
        TokenKind::Minus => Some(BinOpKind::Sub),
        TokenKind::Star => Some(BinOpKind::Mul),
        TokenKind::Slash => Some(BinOpKind::Div),
        TokenKind::Percent => Some(BinOpKind::Mod),
        TokenKind::Pow => Some(BinOpKind::Pow),

        // String concatenation
        TokenKind::Dot => Some(BinOpKind::Concat),

        // Logical
        TokenKind::AndAnd => Some(BinOpKind::AndAnd),
        TokenKind::OrOr => Some(BinOpKind::OrOr),
        TokenKind::NullCoalesce => Some(BinOpKind::NullCoalesce),
        TokenKind::KwOr => Some(BinOpKind::Or),
        TokenKind::KwXor => Some(BinOpKind::Xor),
        TokenKind::KwAnd => Some(BinOpKind::And),

        // Bitwise / Shifts
        TokenKind::Amp => Some(BinOpKind::BitAnd),
        TokenKind::Pipe => Some(BinOpKind::BitOr),
        TokenKind::Caret => Some(BinOpKind::BitXor),
        TokenKind::Shl => Some(BinOpKind::Shl),
        TokenKind::Shr => Some(BinOpKind::Shr),

        // Comparison
        TokenKind::StrictEq => Some(BinOpKind::CmpEqStrict),
        TokenKind::StrictNe => Some(BinOpKind::CmpNeStrict),
        TokenKind::EqEq => Some(BinOpKind::CmpEq),
        TokenKind::NotEq => Some(BinOpKind::CmpNe),

        // Ordering
        TokenKind::Lt => Some(BinOpKind::CmpLt),
        TokenKind::Gt => Some(BinOpKind::CmpGt),
        TokenKind::Le => Some(BinOpKind::CmpLe),
        TokenKind::Ge => Some(BinOpKind::CmpGe),

        // Spaceship
        TokenKind::Spaceship => Some(BinOpKind::CmpSpaceship),

        _ => None,
    }
}

#[inline]
pub fn precedence_of(op: BinOpKind) -> u8 {
    match op {
        BinOpKind::Pow => 90,

        BinOpKind::Mul | BinOpKind::Div | BinOpKind::Mod => 80,

        BinOpKind::Add | BinOpKind::Sub => 70,

        BinOpKind::Concat => 60,

        BinOpKind::Shl | BinOpKind::Shr => 55,

        BinOpKind::CmpLt
        | BinOpKind::CmpGt
        | BinOpKind::CmpLe
        | BinOpKind::CmpGe
        | BinOpKind::CmpSpaceship => 50,

        BinOpKind::CmpEq
        | BinOpKind::CmpNe
        | BinOpKind::CmpEqStrict
        | BinOpKind::CmpNeStrict => 45,

        BinOpKind::BitAnd => 40,
        BinOpKind::BitXor => 35,
        BinOpKind::BitOr => 30,

        BinOpKind::AndAnd => 20,
        BinOpKind::NullCoalesce => 15,
        BinOpKind::OrOr => 10,

        BinOpKind::Xor => 8,
        BinOpKind::And => 5,
        BinOpKind::Or => 2,
    }
}
