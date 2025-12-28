use crate::ast::{Block, CatchClause, Ident, Stmt, TypeRef};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::Span;

impl<'src> Parser<'src> {
    pub(super) fn parse_try_catch_stmt(&mut self) -> Option<Stmt> {
        debug_assert!(self.at(TokenKind::KwTry));

        let try_token = self.bump();
        let start_pos = try_token.span.start;
        let mut last_end = try_token.span.end;

        let (try_block, try_end_pos) = self.expect_block_after(last_end);
        last_end = try_end_pos;

        let mut catches: Vec<CatchClause> = Vec::new();
        while self.at(TokenKind::KwCatch) {
            let catch_token = self.bump();
            last_end = catch_token.span.end;

            self.expect_or_err(TokenKind::LParen, &mut last_end);

            let mut types_vec: Vec<TypeRef> = Vec::new();
            loop {
                if let Some(qn) = self.parse_qualified_name() {
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
                self.error(Diagnostic::error_from_code(
                    ParseDiagnosticCode::ExpectedCatchExceptionType, // TODO?
                    Span::at(last_end),
                ));
            }

            let mut var_ident_opt: Option<Ident> = None;
            if self.at(TokenKind::VarIdent) {
                let token = self.bump();
                var_ident_opt = Some(Ident { span: token.span });
                last_end = token.span.end;
            } else if self.eat(TokenKind::Dollar) {
                let dollar_end = self.current_span().start;
                if let Some(var_token) = self.expect_ident_or_err(&mut last_end)
                {
                    var_ident_opt = Some(Ident {
                        span: var_token.span,
                    });
                    last_end = var_token.span.end;
                } else {
                    self.error_and_recover(
                        Diagnostic::error_from_code(
                            ParseDiagnosticCode::ExpectedIdent,
                            Span::at(dollar_end),
                        ),
                        &[TokenKind::RParen],
                    );
                }
            }

            self.expect_or_err(TokenKind::RParen, &mut last_end);

            let (catch_body, catch_end_pos) = self.expect_block_after(last_end);
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
            let (finally_block, finally_end_pos) =
                self.expect_block_after(last_end);
            last_end = finally_end_pos;
            finally_block_opt = Some(finally_block);
        }

        if catches.is_empty() && finally_block_opt.is_none() {
            self.error(Diagnostic::error_from_code(
                ParseDiagnosticCode::expected_tokens([
                    TokenKind::KwCatch,
                    TokenKind::KwFinally,
                ]),
                Span::at(last_end),
            ));
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

    fn expect_block_after(&mut self, last_end_fallback: u32) -> (Block, u32) {
        if let Some(lb) = self.expect(TokenKind::LBrace) {
            match self.parse_block_after_lbrace(lb.span.start) {
                Some(res) => res,
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
            }
        } else {
            self.error(Diagnostic::error_from_code(
                ParseDiagnosticCode::expected_token(TokenKind::LBrace),
                Span::at(last_end_fallback),
            ));
            let span = Span::at(last_end_fallback);
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
