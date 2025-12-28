mod attribute;
mod r#break;
mod class_decl;
mod const_decl;
mod declare;
mod echo;
mod enum_decl;
mod expr_stmt;
mod for_like;
mod function_decl;
mod global;
mod goto;
mod r#if;
mod interface_decl;
mod label;
mod namespace;
mod r#return;
mod switch;
mod throw;
mod trait_decl;
mod try_catch;
mod unset;
mod use_decl;
mod varlike;

use crate::ast::{Block, Stmt};
use crate::parser::Parser;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::TokenKind;
use phynix_core::{Span, Spanned};

impl<'src> Parser<'src> {
    pub fn parse_stmt(&mut self) -> Option<Stmt> {
        if self.at(TokenKind::HtmlChunk) {
            let tok = self.bump();
            return Some(Stmt::HtmlChunk { span: tok.span });
        }

        if self.at(TokenKind::EchoOpen) {
            let tok = self.bump();
            return self.parse_echo_open_stmt(tok.span);
        }

        if self.at(TokenKind::RBrace) {
            return None;
        }

        let _attrs = if self.at(TokenKind::AttrOpen) {
            Some(self.parse_attribute_group_list()?)
        } else {
            None
        };

        if self.at_any(&[
            TokenKind::KwAbstract,
            TokenKind::KwFinal,
            TokenKind::KwReadonly,
            TokenKind::KwClass,
        ]) {
            let save_pos = self.pos;

            let flags = self.parse_class_flags();

            if self.at(TokenKind::KwClass) {
                return self.parse_class_stmt(flags);
            }

            self.pos = save_pos;
            self.skip_trivia_and_cache();
        }

        if self.at(TokenKind::KwConst) {
            return self.parse_const_decl_stmt();
        }

        if self.at(TokenKind::KwDeclare) {
            return self.parse_declare_stmt();
        }

        if self.at(TokenKind::KwDo) {
            return self.parse_do_while_stmt();
        }

        if self.at(TokenKind::KwEcho) {
            return self.parse_echo_stmt();
        }

        if self.at(TokenKind::KwEnum) {
            return self.parse_enum_stmt();
        }

        if self.at(TokenKind::KwFor) {
            return self.parse_for_stmt();
        }

        if self.at(TokenKind::KwForeach) {
            return self.parse_foreach_stmt();
        }

        if self.at(TokenKind::KwFunction) {
            // Anonymous closure as statement: function (...) { ... };
            // or function &(...) { ... };
            return if self.at_nth(1, TokenKind::LParen)
                || (self.at_nth(1, TokenKind::Amp)
                    && self.at_nth(2, TokenKind::LParen))
            {
                self.parse_expr_stmt()
            } else {
                self.parse_function_stmt()
            };
        }

        if self.at(TokenKind::KwGlobal) {
            return self.parse_global_stmt();
        }

        if self.at(TokenKind::KwIf) {
            return self.parse_if_stmt();
        }

        if self.at(TokenKind::KwInterface) {
            return self.parse_interface_stmt();
        }

        if self.at(TokenKind::KwNamespace) {
            return self.parse_namespace_stmt();
        }

        if self.at(TokenKind::KwPrint) {
            return self.parse_expr_stmt();
        }

        if self.at(TokenKind::KwReturn) {
            return self.parse_return_stmt();
        }

        if self.at(TokenKind::KwThrow) {
            return self.parse_throw_stmt();
        }

        if self.at(TokenKind::KwTrait) {
            return self.parse_trait_stmt();
        }

        if self.at(TokenKind::KwTry) {
            return self.parse_try_catch_stmt();
        }

        if self.at(TokenKind::KwUnset) {
            return self.parse_unset_stmt();
        }

        if self.at(TokenKind::KwUse) {
            return self.parse_use_stmt();
        }

        if self.at(TokenKind::KwWhile) {
            return self.parse_while_stmt();
        }

        if self.at(TokenKind::KwSwitch) {
            return self.parse_switch_stmt();
        }

        if let Some(semi_token) = self.expect(TokenKind::Semicolon) {
            return Some(self.parse_empty_stmt(semi_token.span));
        }

        if self.at(TokenKind::KwBreak) {
            return self.parse_break_stmt();
        }

        if self.at(TokenKind::KwContinue) {
            return self.parse_continue_stmt();
        }

        if self.at(TokenKind::Ident) && self.at_nth(1, TokenKind::Colon) {
            return self.parse_label_stmt();
        }

        if self.at(TokenKind::KwGoto) {
            return self.parse_goto_stmt();
        }

        if self.at(TokenKind::KwStatic) && self.at_nth(1, TokenKind::VarIdent) {
            let static_tok = self.bump();
            let start = static_tok.span.start;
            let mut last_end = static_tok.span.end;

            loop {
                if self.at(TokenKind::VarIdent) {
                    let var_tok = self.bump();
                    last_end = var_tok.span.end;

                    if self.eat(TokenKind::Eq) {
                        self.parse_expr_or_err(&mut last_end);
                    }
                } else {
                    self.error(Diagnostic::error_from_code(
                        ParseDiagnosticCode::ExpectedIdent,
                        Span::at(last_end),
                    ));
                    break;
                }

                if self.eat(TokenKind::Comma) {
                    continue;
                }

                break;
            }

            self.expect_or_err(TokenKind::Semicolon, &mut last_end);

            return Some(Stmt::Noop {
                span: Span {
                    start,
                    end: last_end,
                },
            });
        }

        if self.at_any(&[
            TokenKind::KwPublic,
            TokenKind::KwPrivate,
            TokenKind::KwProtected,
            TokenKind::KwStatic,
            TokenKind::KwAbstract,
            TokenKind::KwFinal,
            TokenKind::KwReadonly,
        ]) {
            return None;
        }

        if let Some(stmt) = self.parse_expr_stmt() {
            return Some(stmt);
        }

        None
    }

    fn parse_until_any(&mut self, stop: &[TokenKind]) -> (Block, u32) {
        let start = self.current_span().start;
        let mut items = Vec::new();
        let mut end = start;

        loop {
            if self.eof() || self.at_any(stop) {
                break;
            }

            if self.at(TokenKind::PhpClose) || self.at(TokenKind::HtmlChunk) {
                self.bounce_out_of_php();
                if self.eof() || self.at_any(stop) {
                    break;
                }
                continue;
            }

            let before_pos = self.pos;

            if let Some(stmt) = self.parse_stmt() {
                end = stmt.span().end;
                items.push(stmt);
            } else {
                let sp = self.current_span();
                if !self.eof() {
                    let _ = self.advance();
                }
                end = sp.end;
            }

            if self.pos == before_pos {
                if self.eof() {
                    break;
                }
                let sp = self.current_span();
                let _ = self.advance();
                end = sp.end;
            }
        }

        (
            Block {
                items,
                span: Span { start, end },
            },
            end,
        )
    }

    fn bounce_out_of_php(&mut self) {
        if self.at(TokenKind::PhpClose) {
            let _ = self.bump();
        }
        while self.at(TokenKind::HtmlChunk) {
            let _ = self.bump();
        }
        if self.at(TokenKind::PhpOpen) {
            let _ = self.bump();
        }
    }
}
