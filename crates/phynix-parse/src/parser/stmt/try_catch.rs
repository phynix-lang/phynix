use crate::ast::{Block, CatchClause, Ident, Stmt, TypeRef};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::Span;
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_try_catch_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwTry));

        let try_token = self.bump();
        let start_pos = try_token.span.start;
        let mut last_end = try_token.span.end;

        let (try_block, try_end_pos) = self.expect_block_after(
            ParseDiagnosticCode::ExpectedToken,
            "expected '{' after 'try'",
            last_end,
        );
        last_end = try_end_pos;

        let mut catches: Vec<CatchClause> = Vec::new();
        while self.at(TokenKind::KwCatch) {
            let catch_token = self.bump();
            last_end = catch_token.span.end;

            let lp =
                self.expect(TokenKind::LParen, "expected '(' after 'catch'");
            if let Some(lp) = lp.as_ref() {
                last_end = lp.span.end;
            }

            let mut types_vec: Vec<TypeRef> = Vec::new();
            loop {
                if let Some(qn) =
                    self.parse_qualified_name("expected exception type")
                {
                    let ty_span = qn.span;
                    types_vec.push(TypeRef::Named {
                        name: qn,
                        span: ty_span,
                    });
                    last_end = ty_span.end;
                    if self.eat(TokenKind::Pipe) {
                        continue;
                    }
                }
                break;
            }
            if types_vec.is_empty() {
                self.error_here(
                    ParseDiagnosticCode::ExpectedCatchExceptionType,
                    "expected at least one exception type in catch(...)",
                );
            }

            let mut var_ident_opt: Option<Ident> = None;
            if self.at(TokenKind::VarIdent) {
                let token = self.bump();
                var_ident_opt = Some(Ident { span: token.span });
                last_end = token.span.end;
            } else if self.eat(TokenKind::Dollar) {
                if let Some(var_token) =
                    self.expect_ident("expected variable name in catch()")
                {
                    var_ident_opt = Some(Ident {
                        span: var_token.span,
                    });
                    last_end = var_token.span.end;
                } else {
                    self.error_and_recover(
                        "expected variable name in catch()",
                        &[TokenKind::RParen],
                    );
                }
            }

            let rp = self
                .expect(TokenKind::RParen, "expected ')' after catch header");
            if let Some(rp) = rp.as_ref() {
                last_end = rp.span.end;
            }

            let (catch_body, catch_end_pos) = self.expect_block_after(
                ParseDiagnosticCode::ExpectedToken,
                "expected '{' after catch(...)",
                last_end,
            );
            last_end = catch_end_pos;

            let catch_span = Span {
                start: catch_token.span.start,
                end: catch_end_pos,
            };
            catches.push(CatchClause {
                exception_types: types_vec,
                var: var_ident_opt,
                body: catch_body,
                span: catch_span,
            });
        }

        let mut finally_block_opt = None;
        if self.at(TokenKind::KwFinally) {
            let fin_token = self.bump();
            last_end = fin_token.span.end;
            let (finally_block, finally_end_pos) = self.expect_block_after(
                ParseDiagnosticCode::ExpectedToken,
                "expected '{' after 'finally'",
                last_end,
            );
            last_end = finally_end_pos;
            finally_block_opt = Some(finally_block);
        }

        // PHP requires at least one catch or finally, but for recovery we still produce the node.
        if catches.is_empty() && finally_block_opt.is_none() {
            self.error_here(
                ParseDiagnosticCode::ExpectedCatchOrFinallyAfterTry,
                "expected at least one 'catch' or 'finally' after 'try' block",
            );
        }

        let full_span = Span {
            start: start_pos,
            end: last_end,
        };

        Some(Stmt::Try {
            try_block,
            catches,
            finally_block: finally_block_opt,
            span: full_span,
        })
    }

    fn expect_block_after(
        &mut self,
        code: ParseDiagnosticCode,
        msg: &'static str,
        last_end_fallback: u32,
    ) -> (Block, u32) {
        if let Some(lb) = self.expect(TokenKind::LBrace, msg) {
            match self.parse_block_after_lbrace(lb.span.start) {
                Some(res) => res,
                None => {
                    let span = Span {
                        start: lb.span.start,
                        end: lb.span.start,
                    };
                    (
                        Block {
                            items: Vec::new(),
                            span,
                        },
                        span.end,
                    )
                },
            }
        } else {
            self.error_here(code, msg);
            let span = Span {
                start: last_end_fallback,
                end: last_end_fallback,
            };
            (
                Block {
                    items: Vec::new(),
                    span,
                },
                span.end,
            )
        }
    }
}
