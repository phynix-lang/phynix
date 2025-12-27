mod expr;
mod name;
mod stmt;
mod util;

use crate::ast::*;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::{LanguageKind, Span, Spanned, Strictness};
use phynix_lex::{Token, TokenKind};

pub struct Parser<'src> {
    src: &'src str,
    tokens: &'src [Token],
    len: usize,
    pos: usize,
    cur: &'src Token,
    diagnostics: Vec<Diagnostic>,

    lang: LanguageKind,
    strictness: Strictness,
}

impl<'src> Parser<'src> {
    #[inline(always)]
    pub fn new(
        src: &'src str,
        tokens: &'src [Token],
        lang: LanguageKind,
        strictness: Strictness,
    ) -> Self {
        debug_assert!(tokens
            .last()
            .map(|t| matches!(&t.kind, TokenKind::Eof))
            .unwrap_or(true));

        let mut parser = Parser {
            src,
            tokens,
            len: tokens.len(),
            pos: 0,
            cur: &tokens[0],
            diagnostics: Vec::new(),
            lang,
            strictness,
        };
        parser.skip_trivia_and_cache();
        parser
    }

    pub fn parse_script(mut self) -> (Script, Vec<Diagnostic>) {
        if self.at(TokenKind::PhpOpen) {
            let _open_token = self.bump();
        }

        let start_open = self.current_span();

        let mut items = Vec::new();
        while !self.eof() {
            while self.at(TokenKind::HtmlChunk) {
                self.bump();
            }

            if self.at(TokenKind::PhpOpen) {
                self.bump();
                continue;
            }
            if self.at(TokenKind::PhpClose) {
                self.bump();
                continue;
            }

            if self.eof() {
                break;
            }

            if let Some(stmt) = self.parse_stmt() {
                items.push(stmt);
            } else {
                if self.at(TokenKind::PhpOpen) || self.at(TokenKind::PhpClose) {
                    continue;
                }
                self.recover_one_token("unexpected token at top level");
            }
        }

        let end_span = if let Some(last) = items.last() {
            last.span()
        } else {
            start_open
        };

        let script = Script {
            items,
            span: Span {
                start: start_open.start,
                end: end_span.end,
            },
        };

        (script, self.diagnostics)
    }

    #[inline(always)]
    fn advance(&mut self) -> &'src Token {
        let token = self.cur;
        self.pos += 1;
        while self.pos < self.len && self.tokens[self.pos].kind.is_trivia() {
            self.pos += 1;
        }
        let idx = if self.pos >= self.len {
            self.len - 1
        } else {
            self.pos
        };
        self.cur = &self.tokens[idx];
        token
    }

    #[inline(always)]
    fn kind(&self) -> &TokenKind {
        &self.cur.kind
    }

    #[inline(always)]
    fn span(&self) -> Span {
        self.cur.span
    }

    #[inline(always)]
    pub fn nth_kind(&self, mut n: usize) -> &TokenKind {
        let mut i = self.pos;
        while i < self.len {
            let k = &self.tokens[i].kind;
            if !k.is_trivia() {
                if n == 0 {
                    return k;
                }
                n -= 1;
            }
            i += 1;
        }
        &TokenKind::Eof
    }

    #[inline(always)]
    pub fn at(&self, k: TokenKind) -> bool {
        matches!(self.kind(), x if *x == k)
    }

    #[inline(always)]
    pub fn at_nth(&self, n: usize, k: TokenKind) -> bool {
        matches!(self.nth_kind(n), x if *x == k)
    }

    #[inline(always)]
    pub fn at_any(&self, ks: &[TokenKind]) -> bool {
        let k = self.kind();
        ks.iter().any(|x| *x == *k)
    }

    #[inline(always)]
    pub fn eat(&mut self, k: TokenKind) -> bool {
        if self.at(k) {
            let _ = self.bump();
            true
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn bump(&mut self) -> &'src Token {
        self.advance()
    }

    #[inline(always)]
    pub fn eof(&self) -> bool {
        matches!(self.kind(), &TokenKind::Eof)
    }

    #[inline(always)]
    pub fn current_span(&self) -> Span {
        self.span()
    }

    #[inline(always)]
    pub fn prev_span(&self) -> Option<Span> {
        if self.pos == 0 {
            None
        } else {
            Some(self.tokens[self.pos - 1].span)
        }
    }

    #[inline(always)]
    fn is_ident_like_kw(&self, k: &TokenKind) -> bool {
        matches!(
            k,
            TokenKind::KwAbstract
                | TokenKind::KwAnd
                | TokenKind::KwArray
                | TokenKind::KwAs
                | TokenKind::KwBreak
                | TokenKind::KwCallable
                | TokenKind::KwCase
                | TokenKind::KwCatch
                | TokenKind::KwClass
                | TokenKind::KwClone
                | TokenKind::KwConst
                | TokenKind::KwContinue
                | TokenKind::KwDeclare
                | TokenKind::KwDefault
                | TokenKind::KwDie
                | TokenKind::KwDo
                | TokenKind::KwEcho
                | TokenKind::KwElse
                | TokenKind::KwElseIf
                | TokenKind::KwEmpty
                | TokenKind::KwEndDeclare
                | TokenKind::KwEndFor
                | TokenKind::KwEndForeach
                | TokenKind::KwEndIf
                | TokenKind::KwEndSwitch
                | TokenKind::KwEndWhile
                | TokenKind::KwEnum
                | TokenKind::KwEval
                | TokenKind::KwExit
                | TokenKind::KwExtends
                | TokenKind::KwFinal
                | TokenKind::KwFinally
                | TokenKind::KwFn
                | TokenKind::KwFor
                | TokenKind::KwForeach
                | TokenKind::KwFrom
                | TokenKind::KwFunction
                | TokenKind::KwGlobal
                | TokenKind::KwGoto
                | TokenKind::KwIf
                | TokenKind::KwImplements
                | TokenKind::KwInclude
                | TokenKind::KwIncludeOnce
                | TokenKind::KwInstanceof
                | TokenKind::KwInsteadof
                | TokenKind::KwInterface
                | TokenKind::KwIsset
                | TokenKind::KwList
                | TokenKind::KwMatch
                | TokenKind::KwNamespace
                | TokenKind::KwNew
                | TokenKind::KwOr
                | TokenKind::KwParent
                | TokenKind::KwPrint
                | TokenKind::KwPrivate
                | TokenKind::KwProtected
                | TokenKind::KwPublic
                | TokenKind::KwReadonly
                | TokenKind::KwRequire
                | TokenKind::KwRequireOnce
                | TokenKind::KwReturn
                | TokenKind::KwSelf
                | TokenKind::KwStatic
                | TokenKind::KwSwitch
                | TokenKind::KwThrow
                | TokenKind::KwTrait
                | TokenKind::KwTry
                | TokenKind::KwUnset
                | TokenKind::KwUse
                | TokenKind::KwVar
                | TokenKind::KwWhile
                | TokenKind::KwXor
                | TokenKind::KwYield
        )
    }

    pub fn expect(
        &mut self,
        kind: TokenKind,
        msg: &'static str,
    ) -> Option<&'src Token> {
        if self.at(kind) {
            Some(self.bump())
        } else {
            self.error_here(ParseDiagnosticCode::ExpectedToken, msg);
            None
        }
    }

    pub fn expect_ident(&mut self, msg: &'static str) -> Option<&'src Token> {
        if let Some(tok) = self.bump_ident_like() {
            Some(tok)
        } else {
            self.error_here(ParseDiagnosticCode::ExpectedIdent, msg);
            None
        }
    }

    #[inline(always)]
    fn skip_trivia(&mut self) {
        while self.pos < self.len && self.tokens[self.pos].kind.is_trivia() {
            self.pos += 1;
        }
    }

    #[inline(always)]
    fn skip_trivia_and_cache(&mut self) {
        while self.pos < self.len && self.tokens[self.pos].kind.is_trivia() {
            self.pos += 1;
        }
        let idx = if self.pos >= self.len {
            self.len - 1
        } else {
            self.pos
        };
        self.cur = &self.tokens[idx];
    }

    #[inline(always)]
    pub fn peek(&mut self) -> Option<&Token> {
        self.skip_trivia();
        self.tokens.get(self.pos)
    }

    #[inline]
    pub fn slice(&self, token: &Token) -> &'src str {
        debug_assert!(
            self.src.is_char_boundary(token.span.start as usize)
                && self.src.is_char_boundary(token.span.end as usize)
        );

        &self.src[token.span.start as usize..token.span.end as usize]
    }

    #[cold]
    pub fn error_here(&mut self, code: ParseDiagnosticCode, msg: &'static str) {
        let span = if !self.eof() {
            self.span()
        } else {
            self.prev_span().unwrap_or(Span::EMPTY)
        };
        self.error_span(code, span, msg);
    }

    #[cold]
    pub fn error_span(
        &mut self,
        code: ParseDiagnosticCode,
        span: Span,
        message: &'static str,
    ) {
        self.diagnostics
            .push(Diagnostic::error(code, span, message));
    }

    #[cold]
    pub fn recover_one_token(&mut self, message: &'static str) {
        self.error_here(ParseDiagnosticCode::UnexpectedToken, message);
        if !self.eof() {
            let _ = self.advance();
        }
    }

    #[cold]
    pub fn recover_to_any(&mut self, sync: &[TokenKind]) {
        while !self.eof() && !self.at_any(sync) {
            let _ = self.advance();
        }
    }

    pub fn error_and_recover(
        &mut self,
        message: &'static str,
        sync: &[TokenKind],
    ) {
        self.error_here(ParseDiagnosticCode::UnexpectedToken, message);
        self.recover_to_any(sync);
    }
}
