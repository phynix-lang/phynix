use crate::ast::Stmt;
use crate::parser::Parser;
use phynix_core::Span;
use phynix_lex::TokenKind;

impl<'src> Parser<'src> {
    pub(super) fn parse_global_stmt(&mut self) -> Option<Stmt> {
        assert!(self.at(TokenKind::KwGlobal));

        let global_token = self.bump();
        let mut end = global_token.span.end;

        loop {
            if self.at(TokenKind::VarIdent) {
                end = self.bump().span.end;
            } else if self.eat(TokenKind::Dollar) {
                if let Some(id) =
                    self.expect_ident("expected variable name after '$'")
                {
                    end = id.span.end;
                } else {
                    self.error_and_recover(
                        "expected variable name after '$'",
                        &[TokenKind::Comma, TokenKind::Semicolon],
                    );
                }
            } else {
                self.error_here("expected variable after 'global'");
                self.recover_to_any(&[TokenKind::Semicolon]);
                break;
            }

            if self.eat(TokenKind::Comma) {
                end = self.prev_span().unwrap().end;
                continue;
            }
            break;
        }

        if let Some(semi) =
            self.expect(TokenKind::Semicolon, "expected ';' after global list")
        {
            end = semi.span.end;
        } else {
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
