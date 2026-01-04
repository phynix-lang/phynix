use crate::ast::{
    BinOpKind, ClassNameRef, Expr, QualifiedName, SpecialClassName,
};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

impl<'src> Parser<'src> {
    pub(super) fn parse_binop_prec_impl(
        &mut self,
        min_prec: u8,
    ) -> Option<Expr> {
        let left = self.parse_prefix_term()?;
        Some(self.parse_binop_prec_from_left(left, min_prec))
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

            if prec == min_prec && is_non_associative(op_kind) {
                let cur_span = self.current_span();
                self.error(Diagnostic::error(
                    ParseDiagnosticCode::UnexpectedToken,
                    cur_span,
                    format!(
                        "operator '{}' is non-associative",
                        op_kind_to_str(op_kind)
                    ),
                ));
                break;
            }

            if op_kind == BinOpKind::InstanceOf {
                left = self.parse_instanceof_from_left(left);
                continue;
            }

            let (l, r) = match self.bump_binop_and_parse_right(left, op_kind) {
                Ok(v) => v,
                Err(e) => return e,
            };

            let span = Span {
                start: l.span().start,
                end: r.span().end,
            };
            left = if op_kind == BinOpKind::NullCoalesce {
                Expr::NullCoalesce {
                    left: Box::new(l),
                    right: Box::new(r),
                    span,
                }
            } else {
                Expr::BinaryOp {
                    op: op_kind,
                    left: Box::new(l),
                    right: Box::new(r),
                    span,
                }
            };
        }

        left
    }

    fn parse_instanceof_from_left(&mut self, left: Expr) -> Expr {
        let instanceof_token = self.bump();
        let last_end = instanceof_token.span.end;

        let class_ref;
        if self.at(TokenKind::Backslash) || self.at(TokenKind::Ident) {
            match self.parse_qualified_name() {
                Some(qn) => class_ref = ClassNameRef::Qualified(qn),
                None => {
                    self.recover_to_any(&[
                        TokenKind::Semicolon,
                        TokenKind::Comma,
                        TokenKind::RParen,
                        TokenKind::RBracket,
                    ]);

                    let span = Span {
                        start: left.span().start,
                        end: last_end,
                    };

                    return Expr::InstanceOf {
                        expr: Box::new(left),
                        class: ClassNameRef::Qualified(QualifiedName {
                            absolute: false,
                            parts: vec![],
                            span,
                        }),
                        span,
                    };
                },
            }
        } else if (self.at(TokenKind::KwSelf)
            || self.at(TokenKind::KwParent)
            || self.at(TokenKind::KwStatic))
            && self.at_nth(1, TokenKind::ColCol)
        {
            class_ref = self.parse_instanceof_dynamic_class_ref();
        } else if self.at(TokenKind::KwSelf) {
            let token = self.bump();
            class_ref =
                ClassNameRef::Special(SpecialClassName::SelfType(token.span));
        } else if self.at(TokenKind::KwParent) {
            let token = self.bump();
            class_ref =
                ClassNameRef::Special(SpecialClassName::ParentType(token.span));
        } else if self.at(TokenKind::KwStatic) {
            let token = self.bump();
            class_ref =
                ClassNameRef::Special(SpecialClassName::StaticType(token.span));
        } else {
            match self.parse_prefix_term() {
                Some(expr) => {
                    let base = self.parse_postfix_chain(expr).unwrap();
                    class_ref = ClassNameRef::Dynamic(Box::new(base));
                },
                None => {
                    self.error_and_recover(
                        Diagnostic::error_from_code(
                            ParseDiagnosticCode::ExpectedIdent,
                            Span::at(last_end),
                        ),
                        &[
                            TokenKind::Semicolon,
                            TokenKind::Comma,
                            TokenKind::RParen,
                            TokenKind::RBracket,
                        ],
                    );

                    let span = Span {
                        start: left.span().start,
                        end: last_end,
                    };

                    return Expr::InstanceOf {
                        expr: Box::new(left),
                        class: ClassNameRef::Qualified(QualifiedName {
                            absolute: false,
                            parts: vec![],
                            span,
                        }),
                        span,
                    };
                },
            }
        };

        let class_span = match &class_ref {
            ClassNameRef::Qualified(qn) => qn.span,
            ClassNameRef::Special(special) => special.span(),
            ClassNameRef::Dynamic(expr) => expr.span(),
        };

        let span = Span {
            start: left.span().start,
            end: class_span.end,
        };
        Expr::InstanceOf {
            expr: Box::new(left),
            class: class_ref,
            span,
        }
    }

    #[inline]
    fn bump_binop_and_parse_right(
        &mut self,
        left: Expr,
        op_kind: BinOpKind,
    ) -> Result<(Expr, Expr), Expr> {
        let op_token = self.bump();

        let mut right = match self.parse_prefix_term() {
            Some(expr) => expr,
            None => {
                let err_span = Span::at(op_token.span.end);
                self.error_and_recover(
                    Diagnostic::error_from_code(
                        ParseDiagnosticCode::ExpectedExpression,
                        err_span,
                    ),
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
                    end: op_token.span.end,
                };
                return Err(Expr::BinaryOp {
                    op: op_kind,
                    left: Box::new(left),
                    right: Box::new(Expr::Error { span: err_span }),
                    span,
                });
            },
        };

        let prec = precedence_of(op_kind);
        right = self.maybe_recurse_higher_prec(
            right,
            prec,
            is_right_associative(op_kind),
        );

        Ok((left, right))
    }

    #[inline]
    fn maybe_recurse_higher_prec(
        &mut self,
        right: Expr,
        prec: u8,
        right_assoc: bool,
    ) -> Expr {
        if self.higher_prec_follows(prec, right_assoc) {
            self.parse_binop_prec_from_left(
                right,
                if right_assoc { prec } else { prec + 1 },
            )
        } else {
            right
        }
    }

    #[inline]
    fn peek_binop(&self) -> Option<BinOpKind> {
        token_to_binop(self.nth_kind(0))
    }

    #[inline]
    fn higher_prec_follows(&self, cur_prec: u8, right_assoc: bool) -> bool {
        self.peek_binop()
            .map(|k| {
                let next_prec = precedence_of(k);
                if right_assoc {
                    next_prec >= cur_prec
                } else {
                    next_prec > cur_prec
                }
            })
            .unwrap_or(false)
    }

    fn parse_instanceof_dynamic_class_ref(&mut self) -> ClassNameRef {
        let start_pos = self.current_span().start;
        match self.parse_prefix_term() {
            Some(expr) => {
                let base = self.parse_postfix_chain(expr).unwrap();
                ClassNameRef::Dynamic(Box::new(base))
            },
            None => {
                self.error(Diagnostic::error_from_code(
                    ParseDiagnosticCode::ExpectedExpression,
                    Span::at(start_pos),
                ));
                ClassNameRef::Qualified(QualifiedName {
                    absolute: false,
                    parts: vec![],
                    span: Span::at(start_pos),
                })
            },
        }
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
        TokenKind::NotEq | TokenKind::NotEqAlt => Some(BinOpKind::CmpNe),

        // Ordering
        TokenKind::Lt => Some(BinOpKind::CmpLt),
        TokenKind::Gt => Some(BinOpKind::CmpGt),
        TokenKind::Le => Some(BinOpKind::CmpLe),
        TokenKind::Ge => Some(BinOpKind::CmpGe),

        // Spaceship
        TokenKind::Spaceship => Some(BinOpKind::CmpSpaceship),

        // Type
        TokenKind::KwInstanceof => Some(BinOpKind::InstanceOf),

        _ => None,
    }
}

#[inline]
pub fn precedence_of(op: BinOpKind) -> u8 {
    match op {
        BinOpKind::Pow => 100,

        BinOpKind::InstanceOf => 90,

        BinOpKind::Mul | BinOpKind::Div | BinOpKind::Mod => 80,

        BinOpKind::Add | BinOpKind::Sub | BinOpKind::Concat => 70,

        BinOpKind::Shl | BinOpKind::Shr => 60,

        BinOpKind::CmpLt
        | BinOpKind::CmpGt
        | BinOpKind::CmpLe
        | BinOpKind::CmpGe => 50,

        BinOpKind::CmpEq
        | BinOpKind::CmpNe
        | BinOpKind::CmpEqStrict
        | BinOpKind::CmpNeStrict
        | BinOpKind::CmpSpaceship => 45,

        BinOpKind::BitAnd => 40,
        BinOpKind::BitXor => 35,
        BinOpKind::BitOr => 30,

        BinOpKind::AndAnd => 20,
        BinOpKind::OrOr => 10,
        BinOpKind::NullCoalesce => 8,

        BinOpKind::Xor => 5,
        BinOpKind::And => 4,
        BinOpKind::Or => 2,
    }
}

#[inline]
pub fn is_right_associative(op: BinOpKind) -> bool {
    matches!(op, BinOpKind::Pow | BinOpKind::NullCoalesce)
}

#[inline]
fn op_kind_to_str(op: BinOpKind) -> &'static str {
    match op {
        BinOpKind::Add => "+",
        BinOpKind::Sub => "-",
        BinOpKind::Mul => "*",
        BinOpKind::Div => "/",
        BinOpKind::Mod => "%",
        BinOpKind::Pow => "**",
        BinOpKind::Concat => ".",
        BinOpKind::BitAnd => "&",
        BinOpKind::BitOr => "|",
        BinOpKind::BitXor => "^",
        BinOpKind::Shl => "<<",
        BinOpKind::Shr => ">>",
        BinOpKind::AndAnd => "&&",
        BinOpKind::OrOr => "||",
        BinOpKind::And => "and",
        BinOpKind::Or => "or",
        BinOpKind::Xor => "xor",
        BinOpKind::CmpEq => "==",
        BinOpKind::CmpNe => "!=",
        BinOpKind::CmpEqStrict => "===",
        BinOpKind::CmpNeStrict => "!==",
        BinOpKind::CmpLt => "<",
        BinOpKind::CmpLe => "<=",
        BinOpKind::CmpGt => ">",
        BinOpKind::CmpGe => ">=",
        BinOpKind::CmpSpaceship => "<=>",
        BinOpKind::NullCoalesce => "??",
        BinOpKind::InstanceOf => "instanceof",
    }
}

#[inline]
pub fn is_non_associative(op: BinOpKind) -> bool {
    matches!(
        op,
        BinOpKind::CmpLt
            | BinOpKind::CmpGt
            | BinOpKind::CmpLe
            | BinOpKind::CmpGe
            | BinOpKind::CmpEq
            | BinOpKind::CmpNe
            | BinOpKind::CmpEqStrict
            | BinOpKind::CmpNeStrict
            | BinOpKind::CmpSpaceship
            | BinOpKind::InstanceOf
    )
}
