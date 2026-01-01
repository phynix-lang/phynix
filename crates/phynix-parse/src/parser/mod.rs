mod expr;
mod name;
mod stmt;
mod util;

use crate::ast::*;
use phynix_core::diagnostics::parser::ParseDiagnosticCode;
use phynix_core::diagnostics::Diagnostic;
use phynix_core::token::{Token, TokenKind};
use phynix_core::{LanguageKind, PhynixConfig, Span, Spanned};

pub struct Parser<'src> {
    src: &'src str,
    tokens: &'src [Token],
    len: usize,
    pos: usize,
    cur: &'src Token,
    diagnostics: Vec<Diagnostic>,

    _config: PhynixConfig,
}

impl<'src> Parser<'src> {
    #[inline(always)]
    pub fn new(
        src: &'src str,
        tokens: &'src [Token],
        config: PhynixConfig,
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
            _config: config,
        };
        parser.skip_trivia_and_cache();
        parser
    }

    pub fn parse_script(mut self) -> (Script, Vec<Diagnostic>) {
        if matches!(self._config.language, LanguageKind::PhxCode) {
            // Already in PHP mode, no open tag expected at start
        } else if self.at(TokenKind::PhpOpen) || self.at(TokenKind::EchoOpen) {
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
                let sp = self.current_span();
                self.recover_one_token(Diagnostic::error(
                    ParseDiagnosticCode::UnexpectedToken,
                    sp,
                    "unexpected token at top level",
                ));
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
    fn current_span(&self) -> Span {
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

    pub fn expect(&mut self, kind: TokenKind) -> Option<&'src Token> {
        self.at(kind).then(|| self.bump())
    }

    /// Returns `true` and updates `span_end` if token was found.
    /// Emits error at `span_end` position if not found.
    pub fn expect_or_err(
        &mut self,
        kind: TokenKind,
        span_end: &mut u32,
    ) -> bool {
        if let Some(token) = self.expect(kind) {
            *span_end = token.span.end;
            true
        } else {
            self.error(Diagnostic::error_from_code(
                ParseDiagnosticCode::expected_token(kind),
                Span::at(*span_end),
            ));
            false
        }
    }

    pub fn expect_ident(&mut self) -> Option<&'src Token> {
        match self.kind() {
            TokenKind::Ident => Some(self.bump()),
            k if self.is_ident_like_kw(k) => Some(self.bump()),
            _ => None,
        }
    }

    pub fn expect_ident_or_err(
        &mut self,
        last_end: &mut u32,
    ) -> Option<&'src Token> {
        if let Some(tok) = self.expect_ident() {
            *last_end = tok.span.end;
            Some(tok)
        } else {
            self.error(Diagnostic::error_from_code(
                ParseDiagnosticCode::ExpectedIdent,
                Span::at(*last_end),
            ));
            None
        }
    }

    pub fn expect_ident_ast_or_err(&mut self, last_end: &mut u32) -> Ident {
        if let Some(tok) = self.expect_ident_or_err(last_end) {
            Ident { span: tok.span }
        } else {
            Ident {
                span: Span::at(*last_end),
            }
        }
    }

    pub fn parse_expr_or_err(&mut self, last_end: &mut u32) -> Expr {
        self.parse_or_err(
            ParseDiagnosticCode::ExpectedExpression,
            Span::at(*last_end),
            |p| p.parse_expr(),
        )
        .map(|e| {
            *last_end = e.span().end;
            e
        })
        .unwrap_or_else(|s| Expr::Error { span: s })
    }

    pub fn parse_or_err<T>(
        &mut self,
        code: ParseDiagnosticCode,
        span: Span,
        f: impl FnOnce(&mut Self) -> Option<T>,
    ) -> Result<T, Span> {
        if let Some(res) = f(self) {
            Ok(res)
        } else {
            self.error(Diagnostic::error_from_code(code, span));
            Err(span)
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
    pub fn error(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    #[cold]
    pub fn recover_one_token(&mut self, diagnostic: Diagnostic) {
        self.error(diagnostic);
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

    #[cold]
    pub fn error_and_recover(
        &mut self,
        diagnostic: Diagnostic,
        sync: &[TokenKind],
    ) {
        self.error(diagnostic);
        self.recover_to_any(sync);
    }
}
