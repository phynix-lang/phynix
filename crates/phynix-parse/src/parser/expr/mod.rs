mod binop;
mod call;
mod closure;
mod include;
mod literal;
mod r#match;
mod new;
mod parens;
mod variable;
mod yield_like;

use crate::ast::{
    BinOpKind, CastKind, ClassNameRef, Expr, Ident, QualifiedName, UnOpKind,
};
use crate::parser::Parser;
use phynix_core::{Span, Spanned};
use phynix_lex::{Token, TokenKind};

impl<'src> Parser<'src> {
    fn parse_primary(&mut self) -> Option<Expr> {
        if self.at(TokenKind::Int) {
            return self.parse_int_literal();
        }

        if self.at(TokenKind::Float) {
            return self.parse_float_literal();
        }

        if self.at_any(&[TokenKind::StrSq, TokenKind::StrDq, TokenKind::StrBt])
        {
            return self.parse_string_like_literal();
        }

        if self.at_variable_start() {
            return self.parse_variable_expr();
        }

        if let Some(kind) = self.keyword_to_include_kind(*self.nth_kind(0)) {
            return self.parse_include_expr(kind);
        }

        if self.at_closure_start() {
            return self.parse_closure_expr();
        }

        if self.at(TokenKind::AttrOpen) {
            let _attrs = self.parse_attribute_group_list();

            if self.at_closure_start() {
                return self.parse_closure_expr();
            }
        }

        if self.at(TokenKind::KwNew) {
            return self.parse_new_expr();
        }

        if self.at(TokenKind::KwYield) {
            return self.parse_yield_like_expr();
        }

        if self.at(TokenKind::KwMatch) {
            return self.parse_match_expr();
        }

        if self.eat(TokenKind::LBracket) {
            return self.parse_array_literal(true);
        }

        if self.at(TokenKind::KwArray) {
            return self.parse_array_construct();
        }

        if self.at(TokenKind::Backslash) {
            return self.parse_qualified_expr();
        }

        if self.at(TokenKind::KwStatic) && self.at_nth(1, TokenKind::ColCol) {
            let tok = self.bump();
            let span = tok.span;

            let ident = Ident { span };
            let qn = QualifiedName {
                absolute: false,
                parts: vec![ident],
                span,
            };

            return Some(Expr::ConstFetch { name: qn, span });
        }

        if self.at(TokenKind::Ident) {
            if self.at_any(&[
                TokenKind::KwFor,
                TokenKind::KwForeach,
                TokenKind::KwWhile,
                TokenKind::KwSwitch,
                TokenKind::KwIf,
                TokenKind::KwElse,
                TokenKind::KwReturn,
                TokenKind::KwTry,
                TokenKind::KwCatch,
                TokenKind::KwFinally,
            ]) {
                return None;
            }
            return self.parse_identifier_expr();
        }

        if self.eat(TokenKind::LParen) {
            return self.parse_paren_expr();
        }

        None
    }

    #[inline]
    fn at_closure_start(&self) -> bool {
        (self.at(TokenKind::KwStatic)
            && (self.nth_kind(1) == &TokenKind::KwFn
                || self.nth_kind(1) == &TokenKind::KwFunction))
            || self.at_any(&[TokenKind::KwFn, TokenKind::KwFunction])
    }

    pub(crate) fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_conditional_expr()
    }

    fn parse_conditional_expr(&mut self) -> Option<Expr> {
        let mut cond = self.parse_coalesce_expr()?;

        loop {
            if !self.eat(TokenKind::Question) {
                break;
            }
            let q_span = self.prev_span().unwrap();

            let then_opt = if self.at(TokenKind::Colon) {
                None
            } else {
                match self.parse_coalesce_expr() {
                    Some(expr) => Some(expr),
                    None => {
                        self.error_and_recover(
                            "expected expression after '?'",
                            &[
                                TokenKind::Colon,
                                TokenKind::Semicolon,
                                TokenKind::Comma,
                                TokenKind::RParen,
                                TokenKind::RBracket,
                            ],
                        );
                        None
                    },
                }
            };

            if !self.eat(TokenKind::Colon) {
                self.error_and_recover(
                    "expected ':' in ternary expression",
                    &[
                        TokenKind::Colon,
                        TokenKind::Semicolon,
                        TokenKind::Comma,
                        TokenKind::RParen,
                        TokenKind::RBracket,
                    ],
                );
                let fake = self.prev_span().unwrap_or(q_span);
                return Some(Expr::Error { span: fake });
            }

            let else_expr = match self.parse_conditional_expr() {
                Some(expr) => expr,
                None => {
                    self.error_and_recover(
                        "expected expression after ':' in ternary expression",
                        &[
                            TokenKind::Semicolon,
                            TokenKind::Comma,
                            TokenKind::RParen,
                            TokenKind::RBracket,
                        ],
                    );
                    let fake = self.prev_span().unwrap_or(q_span);
                    Expr::Error { span: fake }
                },
            };

            let span = Span {
                start: cond.span().start,
                end: else_expr.span().end,
            };

            cond = Expr::Ternary {
                condition: Box::new(cond),
                then_expr: then_opt.map(Box::new),
                else_expr: Box::new(else_expr),
                span,
            };
        }

        Some(cond)
    }

    fn parse_coalesce_expr(&mut self) -> Option<Expr> {
        let mut left = self.parse_assignment_expr()?;

        loop {
            if self.eat(TokenKind::NullCoalesce) {
                let right = match self.parse_assignment_expr() {
                    Some(expr) => expr,
                    None => {
                        self.error_and_recover(
                            "expected expression after '??'",
                            &[
                                TokenKind::Comma,
                                TokenKind::Semicolon,
                                TokenKind::RParen,
                                TokenKind::RBracket,
                            ],
                        );
                        break;
                    },
                };

                let span = Span {
                    start: left.span().start,
                    end: right.span().end,
                };

                left = Expr::NullCoalesce {
                    left: Box::new(left),
                    right: Box::new(right),
                    span,
                };

                continue;
            }

            break;
        }

        Some(left)
    }

    fn parse_assignment_expr(&mut self) -> Option<Expr> {
        let left = self.parse_binop_prec(0)?;

        if self.eat(TokenKind::Eq) {
            let rhs = self
                .parse_assignment_expr()
                .or_else(|| self.parse_binop_prec(0))?;
            let span = Span {
                start: left.span().start,
                end: rhs.span().end,
            };
            return Some(Expr::Assign {
                target: Box::new(left),
                value: Box::new(rhs),
                span,
            });
        }

        if self.eat(TokenKind::NullCoalesceAssign) {
            let rhs = self
                .parse_assignment_expr()
                .or_else(|| self.parse_binop_prec(0))?;
            let span = Span {
                start: left.span().start,
                end: rhs.span().end,
            };
            return Some(Expr::CoalesceAssign {
                target: Box::new(left),
                value: Box::new(rhs),
                span,
            });
        }

        let op = match self.nth_kind(0) {
            TokenKind::PlusEq => Some(BinOpKind::Add),
            TokenKind::MinusEq => Some(BinOpKind::Sub),
            TokenKind::MulEq => Some(BinOpKind::Mul),
            TokenKind::DivEq => Some(BinOpKind::Div),
            TokenKind::ModEq => Some(BinOpKind::Mod),
            TokenKind::DotEq => Some(BinOpKind::Concat),
            TokenKind::AmpEq => Some(BinOpKind::BitAnd),
            TokenKind::PipeEq => Some(BinOpKind::BitOr),
            TokenKind::CaretEq => Some(BinOpKind::BitXor),
            TokenKind::ShlEq => Some(BinOpKind::Shl),
            TokenKind::ShrEq => Some(BinOpKind::Shr),
            TokenKind::PowEq => Some(BinOpKind::Pow),
            _ => None,
        };

        if let Some(bin) = op {
            self.bump();
            let rhs = self
                .parse_assignment_expr()
                .or_else(|| self.parse_binop_prec(0))?;
            let span = Span {
                start: left.span().start,
                end: rhs.span().end,
            };
            return Some(Expr::CompoundAssign {
                op: bin,
                target: Box::new(left),
                value: Box::new(rhs),
                span,
            });
        }

        Some(left)
    }

    fn parse_binop_prec(&mut self, min_prec: u8) -> Option<Expr> {
        self.parse_binop_prec_impl(min_prec)
    }

    pub(super) fn parse_prefix_term(&mut self) -> Option<Expr> {
        if self.eat(TokenKind::PlusPlus) {
            let op_span = self.prev_span().unwrap();

            let inner = match self.parse_prefix_term() {
                Some(e) => e,
                None => {
                    self.error_here("expected expression after '++'");
                    return Some(Expr::Error { span: op_span });
                },
            };

            let span = Span {
                start: op_span.start,
                end: inner.span().end,
            };

            return Some(Expr::PrefixInc {
                target: Box::new(inner),
                span,
            });
        }

        if self.eat(TokenKind::MinusMinus) {
            let op_span = self.prev_span().unwrap();

            let inner = match self.parse_prefix_term() {
                Some(e) => e,
                None => {
                    self.error_here("expected expression after '--'");
                    return Some(Expr::Error { span: op_span });
                },
            };

            let span = Span {
                start: op_span.start,
                end: inner.span().end,
            };

            return Some(Expr::PrefixDec {
                target: Box::new(inner),
                span,
            });
        }

        if self.at(TokenKind::KwThrow) {
            let throw_token = self.bump();
            let rhs = match self.parse_prefix_term() {
                Some(e) => e,
                None => {
                    self.error_here("expected expression after 'throw'");
                    let span = throw_token.span;
                    return Some(Expr::Error { span });
                },
            };

            let span = Span {
                start: throw_token.span.start,
                end: rhs.span().end,
            };
            return Some(Expr::Throw {
                expr: Box::new(rhs),
                span,
            });
        }

        if self.at(TokenKind::KwClone) {
            let clone_token = self.bump();
            let inner = self.parse_prefix_term()?;
            let span = Span {
                start: clone_token.span.start,
                end: inner.span().end,
            };
            return Some(Expr::Clone {
                expr: Box::new(inner),
                span,
            });
        }

        if self.at(TokenKind::KwExit) || self.at(TokenKind::KwDie) {
            let kw = self.bump();
            let start = kw.span.start;

            let arg_opt = if self.eat(TokenKind::LParen) {
                if self.eat(TokenKind::RParen) {
                    None
                } else {
                    let expr = self.parse_expr();
                    let _ = self.expect(
                        TokenKind::RParen,
                        "expected ')' after exit/die argument",
                    );
                    expr
                }
            } else if self.at_any(&[
                TokenKind::Semicolon,
                TokenKind::Comma,
                TokenKind::Colon,
                TokenKind::RParen,
                TokenKind::RBracket,
                TokenKind::RBrace,
                TokenKind::Eof,
            ]) {
                None
            } else {
                self.parse_expr()
            };

            let end = arg_opt
                .as_ref()
                .map(|e| e.span().end)
                .unwrap_or(kw.span.end);
            return Some(Expr::Exit {
                arg: arg_opt.map(Box::new),
                span: Span { start, end },
            });
        }

        enum Prefix {
            Unary(UnOpKind, u32),
            Cast(CastKind, Span),
        }

        let mut prefixes: Vec<Prefix> = Vec::new();

        loop {
            let before = self.pos;

            if let Some((ck, sp)) = self.try_parse_cast_prefix() {
                prefixes.push(Prefix::Cast(ck, sp));
                continue;
            }

            if let Some((op, pos)) = self.try_parse_unary_prefix() {
                prefixes.push(Prefix::Unary(op, pos));
                continue;
            }

            if self.pos == before {
                break;
            }
        }

        let mut expr = self.parse_primary()?;
        expr = self.parse_postfix_chain(expr)?;

        while let Some(prefix) = prefixes.pop() {
            match prefix {
                Prefix::Unary(op, start_pos) => {
                    let span = Span {
                        start: start_pos,
                        end: expr.span().end,
                    };
                    expr = Expr::UnaryOp {
                        op,
                        expr: Box::new(expr),
                        span,
                    };
                },
                Prefix::Cast(ck, sp) => {
                    let span = Span {
                        start: sp.start,
                        end: expr.span().end,
                    };
                    expr = Expr::Cast {
                        kind: ck,
                        expr: Box::new(expr),
                        span,
                    };
                },
            }
        }

        Some(expr)
    }

    fn try_parse_cast_prefix(&mut self) -> Option<(CastKind, Span)> {
        let save = self.pos;
        if !self.eat(TokenKind::LParen) {
            return None;
        }
        let lp = self.prev_span().unwrap();

        let ident = match self.peek() {
            Some(token)
                if matches!(
                    token.kind,
                    TokenKind::Ident | TokenKind::KwArray
                ) =>
            {
                let ident_token = self.bump();
                let txt = self.slice(&ident_token).to_ascii_lowercase();
                (ident_token, txt)
            },
            _ => {
                self.pos = save;
                self.skip_trivia_and_cache();
                return None;
            },
        };

        if !self.eat(TokenKind::RParen) {
            self.pos = save;
            self.skip_trivia_and_cache();
            return None;
        }
        let rp = self.prev_span().unwrap();

        let kind = match ident.1.as_str() {
            "int" | "integer" => CastKind::Int,
            "float" | "double" | "real" => CastKind::Float,
            "string" => CastKind::String,
            "bool" | "boolean" => CastKind::Bool,
            "array" => CastKind::Array,
            "object" => CastKind::Object,
            _ => {
                self.pos = save;
                self.skip_trivia_and_cache();
                return None;
            },
        };

        Some((
            kind,
            Span {
                start: lp.start,
                end: rp.end,
            },
        ))
    }

    fn try_parse_unary_prefix(&mut self) -> Option<(UnOpKind, u32)> {
        if self.eat(TokenKind::Silence) {
            let span = self.prev_span().unwrap();
            return Some((UnOpKind::Suppress, span.start));
        }
        if self.eat(TokenKind::Bang) {
            let span = self.prev_span().unwrap();
            return Some((UnOpKind::Not, span.start));
        }
        if self.eat(TokenKind::Minus) {
            let span = self.prev_span().unwrap();
            return Some((UnOpKind::Neg, span.start));
        }
        if self.eat(TokenKind::Plus) {
            let span = self.prev_span().unwrap();
            return Some((UnOpKind::Plus, span.start));
        }
        if self.eat(TokenKind::Amp) {
            let span = self.prev_span().unwrap();
            return Some((UnOpKind::Ref, span.start));
        }
        if self.eat(TokenKind::Tilde) {
            let span = self.prev_span().unwrap();
            return Some((UnOpKind::BitNot, span.start));
        }

        None
    }

    fn parse_postfix_chain(&mut self, mut base: Expr) -> Option<Expr> {
        loop {
            if self.eat(TokenKind::LParen) {
                let lparen_span = self.prev_span().unwrap();

                let (args, rparen_span) =
                    self.parse_call_arguments(lparen_span);

                let call_span = Span {
                    start: base.span().start,
                    end: rparen_span.end,
                };

                base = Expr::FunctionCall {
                    callee: Box::new(base),
                    args,
                    span: call_span,
                };

                continue;
            }

            if self.at_any(&[TokenKind::Arrow, TokenKind::NullsafeArrow]) {
                let _nullsafe = if self.eat(TokenKind::NullsafeArrow) {
                    true
                } else {
                    self.eat(TokenKind::Arrow);
                    false
                };

                if self.at_variable_start() {
                    let prop_expr = match self.parse_variable_expr() {
                        Some(expr) => expr,
                        None => {
                            self.error_and_recover(
                                "expected variable after '->'/$",
                                &[
                                    TokenKind::Semicolon,
                                    TokenKind::Comma,
                                    TokenKind::RParen,
                                    TokenKind::RBracket,
                                    TokenKind::RBrace,
                                ],
                            );
                            return Some(base);
                        },
                    };

                    base = self.finish_dynamic_object_member(base, prop_expr);

                    continue;
                }

                if self.eat(TokenKind::LBrace) {
                    let inner = match self.parse_expr() {
                        Some(e) => e,
                        None => {
                            self.error_and_recover(
                                "expected expression after '{' in '->{...}'",
                                &[TokenKind::RBrace],
                            );
                            let fake = self.prev_span().unwrap_or(base.span());
                            Expr::Error { span: fake }
                        },
                    };
                    let rb = self.expect(
                        TokenKind::RBrace,
                        "expected '}' after '->{expr}'",
                    );
                    let _rb_end =
                        rb.map(|t| t.span.end).unwrap_or(inner.span().end);

                    base = self.finish_dynamic_object_member(base, inner);

                    continue;
                }

                let member_token = if let Some(token) =
                    self.expect_member_after_op_or_sync(false, SYNC_POSTFIX)
                {
                    token
                } else {
                    return Some(base);
                };
                let name_ident = Ident {
                    span: member_token.span,
                };

                base = self.finish_named_member(
                    base,
                    name_ident,
                    |t, m, args, span| Expr::MethodCall {
                        target: Box::new(t),
                        method: m,
                        args,
                        span,
                    },
                    |t, m, span| Expr::PropertyFetch {
                        target: Box::new(t),
                        property: m,
                        span,
                    },
                );

                continue;
            }

            if self.eat(TokenKind::LBracket) {
                if self.eat(TokenKind::RBracket) {
                    let span = Span {
                        start: base.span().start,
                        end: self.prev_span().unwrap().end,
                    };
                    base = Expr::ArrayAppend {
                        array: Box::new(base),
                        span,
                    };
                    continue;
                }
                let index_expr = match self.parse_expr() {
                    Some(expr) => expr,
                    None => {
                        self.error_and_recover(
                            "expected expression inside []",
                            &[TokenKind::RBracket],
                        );

                        if !self.at(TokenKind::RBracket) {
                            self.recover_to_any(&[
                                TokenKind::Semicolon,
                                TokenKind::Comma,
                                TokenKind::RParen,
                                TokenKind::RBracket,
                                TokenKind::RBrace,
                            ]);
                            return Some(base);
                        }

                        let span = self.prev_span().unwrap_or(Span::EMPTY);
                        Expr::Error { span }
                    },
                };

                let rb_token = match self
                    .expect(TokenKind::RBracket, "expected ']' after subscript")
                {
                    Some(token) => token,
                    None => {
                        self.recover_to_any(SYNC_POSTFIX);
                        return Some(base);
                    },
                };

                let span = Span {
                    start: base.span().start,
                    end: rb_token.span.end,
                };

                base = Expr::ArrayIndex {
                    array: Box::new(base),
                    index: Box::new(index_expr),
                    span,
                };

                continue;
            }

            if self.at(TokenKind::PlusPlus) {
                let plus_plus_token = self.bump();

                let span = Span {
                    start: base.span().start,
                    end: plus_plus_token.span.end,
                };

                base = Expr::PostfixInc {
                    target: Box::new(base),
                    span,
                };

                continue;
            }

            if self.at(TokenKind::MinusMinus) {
                let minus_minus_token = self.bump();

                let span = Span {
                    start: base.span().start,
                    end: minus_minus_token.span.end,
                };

                base = Expr::PostfixDec {
                    target: Box::new(base),
                    span,
                };

                continue;
            }

            if self.eat(TokenKind::ColCol) {
                let class_ref = match self.make_class_name_ref(&base) {
                    Some(class_name) => class_name,
                    None => {
                        self.error_and_recover(
                            "expected class name before '::'",
                            &[
                                TokenKind::Semicolon,
                                TokenKind::Comma,
                                TokenKind::RParen,
                                TokenKind::RBracket,
                                TokenKind::RBrace,
                            ],
                        );
                        return Some(base);
                    },
                };

                if self.at(TokenKind::VarIdent) {
                    let var_ident_token = self.bump();
                    let prop_ident = Ident {
                        span: var_ident_token.span,
                    };
                    let full_span = Span {
                        start: base.span().start,
                        end: var_ident_token.span.end,
                    };
                    base = Expr::StaticPropertyFetch {
                        class_name: class_ref,
                        property: prop_ident,
                        span: full_span,
                    };
                    continue;
                }
                if self.eat(TokenKind::Dollar) {
                    let name_token = match self.expect_ident_or_sync(
                        "expected property name after '::$'",
                        SYNC_POSTFIX,
                    ) {
                        Some(token) => token,
                        None => return Some(base),
                    };
                    let prop_ident = Ident {
                        span: name_token.span,
                    };
                    let full_span = Span {
                        start: base.span().start,
                        end: name_token.span.end,
                    };
                    base = Expr::StaticPropertyFetch {
                        class_name: class_ref,
                        property: prop_ident,
                        span: full_span,
                    };
                    continue;
                }

                if self.eat(TokenKind::LBrace) {
                    let inner = match self.parse_expr() {
                        Some(e) => e,
                        None => {
                            self.error_and_recover(
                                "expected expression after '{' in '::{...}'",
                                &[TokenKind::RBrace],
                            );
                            let fake = self.prev_span().unwrap_or(base.span());
                            Expr::Error { span: fake }
                        },
                    };
                    let _rbrace = self.expect(
                        TokenKind::RBrace,
                        "expected '}' after '::{expr}'",
                    );

                    base = self.finish_dynamic_static_member(class_ref, inner);

                    continue;
                }

                let member_token = match self
                    .expect_member_after_op_or_sync(true, SYNC_POSTFIX)
                {
                    Some(token) => token,
                    None => return Some(base),
                };

                let member_ident = Ident {
                    span: member_token.span,
                };

                base = self.finish_named_static_member(class_ref, member_ident);

                continue;
            }

            break;
        }

        Some(base)
    }

    fn make_class_name_ref(&self, expr: &Expr) -> Option<ClassNameRef> {
        match expr {
            Expr::VarRef { name, span } => {
                let class_ident = Ident { span: name.span };

                let qn = QualifiedName {
                    absolute: false,
                    parts: vec![class_ident],
                    span: *span,
                };
                Some(ClassNameRef::Qualified(qn))
            },

            Expr::ConstFetch { name, span } => {
                let parts: Vec<Ident> = name
                    .parts
                    .iter()
                    .map(|id| Ident { span: id.span })
                    .collect();

                Some(ClassNameRef::Qualified(QualifiedName {
                    absolute: name.absolute,
                    parts,
                    span: *span,
                }))
            },

            _ => None,
        }
    }

    #[inline]
    fn expect_ident_or_sync(
        &mut self,
        msg: &'static str,
        sync: &[TokenKind],
    ) -> Option<&'src Token> {
        if let Some(token) = self.expect_ident(msg) {
            Some(token)
        } else {
            self.recover_to_any(sync);
            None
        }
    }

    fn parse_qualified_expr(&mut self) -> Option<Expr> {
        debug_assert!(self.at(TokenKind::Backslash));

        let qn = self.parse_qualified_name("expected name")?;
        let span = qn.span;

        Some(Expr::ConstFetch { name: qn, span })
    }

    #[inline]
    fn expect_member_after_op_or_sync(
        &mut self,
        allow_class_keyword: bool,
        sync: &[TokenKind],
    ) -> Option<&'src Token> {
        if allow_class_keyword && self.at(TokenKind::KwClass) {
            return Some(self.bump());
        }
        if let Some(token) = self.bump_ident_like() {
            return Some(token);
        }
        self.recover_to_any(sync);
        None
    }

    #[inline]
    fn bump_ident_like(&mut self) -> Option<&'src Token> {
        match self.kind() {
            TokenKind::Ident => Some(self.bump()),
            k if self.is_ident_like_kw(k) => Some(self.bump()),
            _ => None,
        }
    }

    fn finish_dynamic_object_member(
        &mut self,
        base: Expr,
        member_expr: Expr,
    ) -> Expr {
        self.finish_dynamic_member(
            base,
            member_expr,
            |t, m, args, span| Expr::DynamicMethodCall {
                target: Box::new(t),
                method_expr: Box::new(m),
                args,
                span,
            },
            |t, m, span| Expr::DynamicPropertyFetch {
                target: Box::new(t),
                property_expr: Box::new(m),
                span,
            },
        )
    }

    fn finish_named_member<FCall, FFetch>(
        &mut self,
        base: Expr,
        member_ident: Ident,
        call_builder: FCall,
        fetch_builder: FFetch,
    ) -> Expr
    where
        FCall: FnOnce(Expr, Ident, Vec<crate::ast::Arg>, Span) -> Expr,
        FFetch: FnOnce(Expr, Ident, Span) -> Expr,
    {
        if self.eat(TokenKind::LParen) {
            let lp = self.prev_span().unwrap();
            let (args, rp) = self.parse_call_arguments(lp);
            let span = Span {
                start: base.span().start,
                end: rp.end,
            };
            call_builder(base, member_ident, args, span)
        } else {
            let span = Span {
                start: base.span().start,
                end: member_ident.span.end,
            };
            fetch_builder(base, member_ident, span)
        }
    }

    fn finish_dynamic_member<FCall, FFetch>(
        &mut self,
        base: Expr,
        dyn_expr: Expr,
        call_builder: FCall,
        fetch_builder: FFetch,
    ) -> Expr
    where
        FCall: FnOnce(Expr, Expr, Vec<crate::ast::Arg>, Span) -> Expr,
        FFetch: FnOnce(Expr, Expr, Span) -> Expr,
    {
        if self.eat(TokenKind::LParen) {
            let lp = self.prev_span().unwrap();
            let (args, rp) = self.parse_call_arguments(lp);
            let span = Span {
                start: base.span().start,
                end: rp.end,
            };
            call_builder(base, dyn_expr, args, span)
        } else {
            let span = Span {
                start: base.span().start,
                end: dyn_expr.span().end,
            };
            fetch_builder(base, dyn_expr, span)
        }
    }

    fn finish_named_static_member(
        &mut self,
        class_ref: ClassNameRef,
        member_ident: Ident,
    ) -> Expr {
        if self.eat(TokenKind::LParen) {
            let lp = self.prev_span().unwrap();
            let (args, rp) = self.parse_call_arguments(lp);
            let span = Span {
                start: member_ident.span.start.saturating_sub(1),
                end: rp.end,
            };
            Expr::StaticCall {
                class: class_ref,
                method: member_ident,
                args,
                span,
            }
        } else {
            let span = Span {
                start: member_ident.span.start.saturating_sub(1),
                end: member_ident.span.end,
            };
            Expr::ClassConstFetch {
                class_name: class_ref,
                constant: member_ident,
                span,
            }
        }
    }

    fn finish_dynamic_static_member(
        &mut self,
        class_ref: ClassNameRef,
        dyn_expr: Expr,
    ) -> Expr {
        if self.eat(TokenKind::LParen) {
            let lp = self.prev_span().unwrap();
            let (args, rp) = self.parse_call_arguments(lp);
            let span = Span {
                start: dyn_expr.span().start.saturating_sub(1),
                end: rp.end,
            };
            Expr::StaticCall {
                class: class_ref,
                method: Ident {
                    span: dyn_expr.span(),
                },
                args,
                span,
            }
        } else {
            let span = Span {
                start: dyn_expr.span().start.saturating_sub(1),
                end: dyn_expr.span().end,
            };
            Expr::ClassConstFetch {
                class_name: class_ref,
                constant: Ident {
                    span: dyn_expr.span(),
                },
                span,
            }
        }
    }
}

pub const SYNC_POSTFIX: &[TokenKind] = &[
    TokenKind::Semicolon,
    TokenKind::Comma,
    TokenKind::RParen,
    TokenKind::RBracket,
    TokenKind::RBrace,
];
