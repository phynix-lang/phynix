use memchr::{memchr, memmem};
use phynix_core::token::{Token, TokenKind};
use phynix_core::{LanguageKind, Span, Strictness};
use thiserror::Error;

#[inline(always)]
fn span_u32(start: usize, end: usize) -> Span {
    debug_assert!(start <= u32::MAX as usize && end <= u32::MAX as usize);
    Span {
        start: start as u32,
        end: end as u32,
    }
}

#[derive(Error, Debug)]
pub enum LexError {
    #[error("lexing failed at byte {0}")]
    At(u32),

    #[error("Unterminated block comment starting at byte {0}")]
    UnterminatedBlock(u32),
}

#[cold]
#[inline(never)]
fn lex_error_at(at: u32) -> LexError {
    LexError::At(at)
}

#[cold]
#[inline(never)]
fn lex_unterminated_block(at: u32) -> LexError {
    LexError::UnterminatedBlock(at)
}

pub struct LexResult {
    pub lang: LanguageKind,
    pub strictness: Strictness,
    pub tokens: Vec<Token>,
}

pub fn lex(
    input: &str,
    lang: LanguageKind,
    strictness: Strictness,
) -> Result<LexResult, LexError> {
    let mut tokens = Vec::new();
    lex_into(input, &lang, &strictness, &mut tokens)?;
    Ok(LexResult {
        lang,
        strictness,
        tokens,
    })
}

pub fn lex_into(
    input: &str,
    lang: &LanguageKind,
    strictness: &Strictness,
    out: &mut Vec<Token>,
) -> Result<(), LexError> {
    // BOM
    let bytes = input.as_bytes();
    let (prefix_len, src) = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        (3, &bytes[3..])
    } else {
        (0, bytes)
    };

    out.clear();
    out.reserve(input.len() / 4);

    let mut lexer = Lexer {
        src,
        i: 0,
        start: 0,
        tokens: out,
        prefix: prefix_len,
        unterminated_block_at: None,
        in_php: false,
    };
    lexer.run()?;
    lexer.tokens.push(Token {
        kind: TokenKind::Eof,
        span: lexer.span_here(),
    });
    Ok(())
}

struct Lexer<'src> {
    src: &'src [u8],
    i: usize,
    start: usize,
    tokens: &'src mut Vec<Token>,
    prefix: usize,
    unterminated_block_at: Option<usize>,
    in_php: bool,
}

macro_rules! take_while_u8 {
    ($self:ident, $pred:expr) => {{
        let s = $self.src;
        let mut i = $self.i;
        while i < s.len() {
            let c = unsafe { *s.get_unchecked(i) };
            if !($pred)(c) {
                break;
            }
            i += 1;
        }
        $self.i = i;
    }};
}

impl<'src> Lexer<'src> {
    fn run(&mut self) -> Result<(), LexError> {
        while self.i < self.src.len() {
            self.start = self.i;

            if !self.in_php {
                let s = self.src;
                if let Some((off, kind, len)) = find_next_php_open(s, self.i) {
                    if off > self.i {
                        self.start = self.i;
                        self.i = off;
                        self.push(TokenKind::HtmlChunk, self.span());
                        continue;
                    }

                    // Open tag starts here
                    self.start = self.i;
                    self.i = off + len;
                    self.push(kind, self.span());
                    self.in_php = true;
                    continue;
                } else {
                    self.start = self.i;
                    self.i = s.len();
                    self.push(TokenKind::HtmlChunk, self.span());
                    break;
                }
            }

            let b = self.peek();

            // PHP Close Tag
            if self.in_php && b == b'?' && self.peek2() == Some(b'>') {
                self.bump();
                self.bump();
                self.push(TokenKind::PhpClose, self.span());
                self.in_php = false;
                continue;
            }

            // Heredoc / Nowdoc
            if self.in_php
                && self.peek() == b'<'
                && self.peek2() == Some(b'<')
                && self.src.get(self.i + 2) == Some(&b'<')
            {
                let start_at = self.start;
                if self.lex_heredoc().is_some() {
                    continue;
                } else {
                    self.i = start_at;
                }
            }

            // Whitespace
            if is_ws(b) {
                self.take_ws();
                self.push(TokenKind::WS, self.span());
                continue;
            }

            if b == b'/' && self.peek2() == Some(b'/') {
                self.i += 2;
                self.bump_to_eol_or_php_close();
                self.push(TokenKind::LineComment, self.span());
                continue;
            }
            if b == b'#' {
                if self.peek2() == Some(b'[') {
                    self.i += 2;
                    self.push(TokenKind::AttrOpen, self.span());
                } else {
                    self.bump_to_eol_or_php_close();
                    self.push(TokenKind::HashComment, self.span());
                }
                continue;
            }
            if b == b'/' && self.peek2() == Some(b'*') {
                let is_doc = self.peek3() == Some(b'*');
                self.i += if is_doc { 3 } else { 2 };
                let start_at = self.start;
                if self.scan_block_comment_end().is_none() {
                    self.unterminated_block_at = Some(self.start);
                }
                self.push(
                    if is_doc {
                        TokenKind::Docblock
                    } else {
                        TokenKind::BlockComment
                    },
                    self.span_from(start_at),
                );
                continue;
            }

            // Comments
            if b == b'"' {
                self.scan_string(b'"');
                self.push(TokenKind::StrDq, self.span());
                continue;
            }
            if b == b'\'' {
                self.scan_string(b'\'');
                self.push(TokenKind::StrSq, self.span());
                continue;
            }
            if b == b'`' {
                self.scan_string(b'`');
                self.push(TokenKind::StrBt, self.span());
                continue;
            }

            // Identifiers / Vars
            if b == b'$' && self.peek2().map(is_ident_start).unwrap_or(false) {
                self.bump();
                self.take_ident_tail();
                self.push(TokenKind::VarIdent, self.span());
                continue;
            }
            if is_ident_start(b) {
                self.take_ident_tail();
                let s = &self.src[self.start..self.i];
                if let Some(kw) = kw_of_bytes(s) {
                    self.push(kw, self.span());
                } else {
                    self.push(TokenKind::Ident, self.span());
                }
                continue;
            }

            // Numbers
            if is_dec_digit(b) {
                let kind = self.scan_number();
                self.push(kind, self.span());
                continue;
            }

            // .4 / .4e2 style floats
            if b == b'.' && self.peek2().map(is_dec_digit).unwrap_or(false) {
                self.i += 1;
                self.take_digits_us();

                if matches!(self.peek(), b'e' | b'E') {
                    let save = self.i;
                    self.i += 1;
                    if matches!(self.peek(), b'+' | b'-') {
                        self.i += 1;
                    }
                    if is_dec_digit(self.peek()) {
                        self.take_digits_us();
                    } else {
                        self.i = save;
                    }
                }

                self.push(TokenKind::Float, self.span());
                continue;
            }

            // Operators / Punctuations
            if let Some(kind) = self.scan_punct() {
                self.push(kind, self.span());
                continue;
            }

            // Unknown byte
            return Err(lex_error_at((self.prefix + self.i) as u32));
        }

        if let Some(at) = self.unterminated_block_at {
            return Err(lex_unterminated_block((self.prefix + at) as u32));
        }

        Ok(())
    }

    #[inline(always)]
    fn scan_string(&mut self, quote: u8) {
        self.bump();
        while self.i < self.src.len() {
            match self.src[self.i] {
                b'\\' => {
                    self.i += 2;
                },
                q if q == quote => {
                    self.i += 1;
                    break;
                },
                _ => self.i += 1,
            }
        }
    }

    #[inline(always)]
    fn scan_number(&mut self) -> TokenKind {
        // Hex/Oct/Bin prefixes
        if self.peek() == b'0' {
            match self.peek2() {
                Some(b'x') | Some(b'X') => {
                    self.i += 2;
                    self.take_hex_us();
                    return TokenKind::Int;
                },
                Some(b'o') | Some(b'O') => {
                    self.i += 2;
                    self.take_oct_us();
                    return TokenKind::Int;
                },
                Some(b'b') | Some(b'B') => {
                    self.i += 2;
                    self.take_bin_us();
                    return TokenKind::Int;
                },
                _ => {},
            }
        }

        // Decimal / Float
        self.take_digits_us();
        let mut kind = TokenKind::Int;
        if self.peek() == b'.'
            && self.peek2().map(is_dec_digit).unwrap_or(false)
        {
            kind = TokenKind::Float;
            self.i += 1;
            self.take_digits_us();
        }
        if matches!(self.peek(), b'e' | b'E') {
            let save = self.i;
            self.i += 1;
            if matches!(self.peek(), b'+' | b'-') {
                self.i += 1;
            }
            if is_dec_digit(self.peek()) {
                kind = TokenKind::Float;
                self.take_digits_us();
            } else {
                self.i = save; // Rollback if no digits after 'e'/'E'
            }
        }
        kind
    }

    #[inline(always)]
    fn scan_punct(&mut self) -> Option<TokenKind> {
        use phynix_core::token::TokenKind::*;

        let s = self.src;
        let i = self.i;
        let b0 = *s.get(i)?;
        let b1 = s.get(i + 1).copied();
        let b2 = s.get(i + 2).copied();

        match (b0, b1, b2) {
            (b'?', Some(b'-'), Some(b'>')) => {
                self.i = i + 3;
                return Some(NullsafeArrow);
            },
            (b'>', Some(b'>'), Some(b'=')) => {
                self.i = i + 3;
                return Some(ShrEq);
            },
            (b'<', Some(b'<'), Some(b'=')) => {
                self.i = i + 3;
                return Some(ShlEq);
            },
            (b'*', Some(b'*'), Some(b'=')) => {
                self.i = i + 3;
                return Some(PowEq);
            },
            (b'?', Some(b'?'), Some(b'=')) => {
                self.i = i + 3;
                return Some(NullCoalesceAssign);
            },
            (b'=', Some(b'='), Some(b'=')) => {
                self.i = i + 3;
                return Some(StrictEq);
            },
            (b'!', Some(b'='), Some(b'=')) => {
                self.i = i + 3;
                return Some(StrictNe);
            },
            (b'<', Some(b'='), Some(b'>')) => {
                self.i = i + 3;
                return Some(Spaceship);
            },
            (b'.', Some(b'.'), Some(b'.')) => {
                self.i = i + 3;
                return Some(Ellipsis);
            },

            (b'<', Some(b'<'), _) => {
                self.i = i + 2;
                return Some(Shl);
            },
            (b'>', Some(b'>'), _) => {
                self.i = i + 2;
                return Some(Shr);
            },
            (b'*', Some(b'*'), _) => {
                self.i = i + 2;
                return Some(Pow);
            },
            (b'?', Some(b'?'), _) => {
                self.i = i + 2;
                return Some(NullCoalesce);
            },
            (b'&', Some(b'&'), _) => {
                self.i = i + 2;
                return Some(AndAnd);
            },
            (b'|', Some(b'|'), _) => {
                self.i = i + 2;
                return Some(OrOr);
            },
            (b':', Some(b':'), _) => {
                self.i = i + 2;
                return Some(ColCol);
            },
            (b'-', Some(b'>'), _) => {
                self.i = i + 2;
                return Some(Arrow);
            },
            (b'=', Some(b'>'), _) => {
                self.i = i + 2;
                return Some(FatArrow);
            },
            (b'=', Some(b'='), _) => {
                self.i = i + 2;
                return Some(EqEq);
            },
            (b'!', Some(b'='), _) => {
                self.i = i + 2;
                return Some(NotEq);
            },
            (b'<', Some(b'>'), _) => {
                self.i = i + 2;
                return Some(NotEqAlt);
            },
            (b'<', Some(b'='), _) => {
                self.i = i + 2;
                return Some(Le);
            },
            (b'>', Some(b'='), _) => {
                self.i = i + 2;
                return Some(Ge);
            },
            (b'+', Some(b'+'), _) => {
                self.i = i + 2;
                return Some(PlusPlus);
            },
            (b'-', Some(b'-'), _) => {
                self.i = i + 2;
                return Some(MinusMinus);
            },
            _ => {},
        }

        match (b0, b1) {
            (b'.', Some(b'=')) => {
                self.i = i + 2;
                return Some(DotEq);
            },
            (b'+', Some(b'=')) => {
                self.i = i + 2;
                return Some(PlusEq);
            },
            (b'-', Some(b'=')) => {
                self.i = i + 2;
                return Some(MinusEq);
            },
            (b'*', Some(b'=')) => {
                self.i = i + 2;
                return Some(MulEq);
            },
            (b'/', Some(b'=')) => {
                self.i = i + 2;
                return Some(DivEq);
            },
            (b'%', Some(b'=')) => {
                self.i = i + 2;
                return Some(ModEq);
            },
            (b'&', Some(b'=')) => {
                self.i = i + 2;
                return Some(AmpEq);
            },
            (b'|', Some(b'=')) => {
                self.i = i + 2;
                return Some(PipeEq);
            },
            (b'^', Some(b'=')) => {
                self.i = i + 2;
                return Some(CaretEq);
            },
            _ => {},
        }

        let k = match self.bump() {
            b'=' => Eq,
            b'<' => Lt,
            b'>' => Gt,
            b'?' => Question,
            b'.' => Dot,
            b'+' => Plus,
            b'-' => Minus,
            b'*' => Star,
            b'/' => Slash,
            b'%' => Percent,
            b'&' => Amp,
            b'|' => Pipe,
            b'^' => Caret,
            b'@' => Silence,
            b'!' => Bang,
            b'~' => Tilde,
            b'{' => LBrace,
            b'}' => RBrace,
            b'(' => LParen,
            b')' => RParen,
            b'[' => LBracket,
            b']' => RBracket,
            b';' => Semicolon,
            b',' => Comma,
            b':' => Colon,
            b'\\' => Backslash,
            b'$' => Dollar,
            _ => return None,
        };
        Some(k)
    }

    #[inline]
    fn push(&mut self, kind: TokenKind, span: Span) {
        self.tokens.push(Token { kind, span });
    }

    #[inline]
    fn span(&self) -> Span {
        span_u32(self.prefix + self.start, self.prefix + self.i)
    }

    #[inline]
    fn span_from(&self, s: usize) -> Span {
        span_u32(self.prefix + s, self.prefix + self.i)
    }

    #[inline]
    fn span_here(&self) -> Span {
        let p = self.prefix + self.i;
        span_u32(p, p)
    }

    #[inline(always)]
    fn peek(&self) -> u8 {
        *self.src.get(self.i).unwrap_or(&0)
    }

    #[inline(always)]
    fn peek2(&self) -> Option<u8> {
        self.src.get(self.i + 1).copied()
    }

    #[inline(always)]
    fn peek3(&self) -> Option<u8> {
        self.src.get(self.i + 2).copied()
    }

    #[inline]
    fn bump(&mut self) -> u8 {
        let b = self.peek();
        if self.i < self.src.len() {
            self.i += 1;
        }
        b
    }

    #[inline(always)]
    fn take_digits_us(&mut self) {
        take_while_u8!(self, |c: u8| (b'0'..=b'9').contains(&c) || c == b'_');
    }

    #[inline(always)]
    fn take_ident_tail(&mut self) {
        take_while_u8!(self, |c: u8| IS_ID_CONT[c as usize]);
    }

    #[inline(always)]
    fn take_hex_us(&mut self) {
        take_while_u8!(self, |c: u8| (b'0'..=b'9').contains(&c)
            || (b'a'..=b'f').contains(&c)
            || (b'A'..=b'F').contains(&c)
            || c == b'_');
    }

    #[inline(always)]
    fn take_oct_us(&mut self) {
        take_while_u8!(self, |c: u8| (b'0'..=b'7').contains(&c) || c == b'_');
    }

    #[inline(always)]
    fn take_bin_us(&mut self) {
        take_while_u8!(self, |c: u8| c == b'0' || c == b'1' || c == b'_');
    }

    #[inline(always)]
    fn take_ws(&mut self) {
        take_while_u8!(self, |c: u8| IS_WS[c as usize]);
    }

    #[inline]
    fn bump_to_eol_or_php_close(&mut self) {
        let s = &self.src[self.i..];
        let mut p = 0usize;
        while p < s.len() {
            let c = unsafe { *s.get_unchecked(p) };
            if c == b'\n' || c == b'\r' {
                break;
            }
            if c == b'?'
                && p + 1 < s.len()
                && unsafe { *s.get_unchecked(p + 1) } == b'>'
            {
                break;
            }
            p += 1;
        }
        self.i += p;
    }

    fn lex_heredoc(&mut self) -> Option<()> {
        self.i += 3;

        while let Some(b) = self.src.get(self.i) {
            if *b == b' ' || *b == b'\t' {
                self.i += 1;
            } else {
                break;
            }
        }

        let quoted = matches!(self.peek(), b'\'' | b'"');
        let quote = if quoted { Some(self.bump()) } else { None };

        let label_start = self.i;
        while let Some(b) = self.src.get(self.i) {
            let c = *b;
            let ok = if self.i == label_start {
                (b'A'..=b'Z').contains(&c)
                    || (b'a'..=b'z').contains(&c)
                    || c == b'_'
            } else {
                (b'A'..=b'Z').contains(&c)
                    || (b'a'..=b'z').contains(&c)
                    || (b'0'..=b'9').contains(&c)
                    || c == b'_'
            };
            if !ok {
                break;
            }
            self.i += 1;
        }
        if self.i == label_start {
            return None;
        }
        let label = &self.src[label_start..self.i];

        if let Some(q) = quote {
            if self.peek() == q {
                self.i += 1;
            } else {
                return None;
            }
        }

        while let Some(b) = self.src.get(self.i) {
            let c = *b;
            self.i += 1;
            if c == b'\n' {
                break;
            }
            if c == b'\r' && self.src.get(self.i) == Some(&b'\n') {
                self.i += 1;
                break;
            }
        }

        let mut p = self.i;

        'scan: loop {
            if p >= self.src.len() {
                break 'scan;
            }

            let mut q = p;
            while let Some(b) = self.src.get(q) {
                if *b == b' ' || *b == b'\t' {
                    q += 1;
                } else {
                    break;
                }
            }

            let label_pos = q;
            let mut i_lbl = 0;
            while i_lbl < label.len()
                && self.src.get(label_pos + i_lbl) == Some(&label[i_lbl])
            {
                i_lbl += 1;
            }

            if i_lbl == label.len() {
                let string_end = p;

                let after_label = label_pos + label.len();
                self.i = after_label;

                if self.src.get(self.i) == Some(&b'\r') {
                    self.i += 1;
                }
                if self.src.get(self.i) == Some(&b'\n') {
                    self.i += 1;
                }

                let span = Span {
                    start: (self.prefix + self.start) as u32,
                    end: (self.prefix + string_end) as u32,
                };
                self.push(TokenKind::StrDq, span);
                return Some(());
            }

            if let Some(nl) = memchr(b'\n', &self.src[p..]) {
                p = p + nl + 1;
            } else {
                break 'scan;
            }
        }

        self.i = self.src.len();
        let span = self.span_from(self.start);
        self.push(TokenKind::StrDq, span);
        Some(())
    }

    // Returns index after closing */ or None
    fn scan_block_comment_end(&mut self) -> Option<()> {
        let hay = &self.src[self.i..];
        let rel = memmem::find(hay, b"*/")?;
        self.i += rel + 2;
        Some(())
    }
}

#[inline]
fn find_next_php_open(
    s: &[u8],
    from: usize,
) -> Option<(usize, TokenKind, usize)> {
    let mut p = from;

    while p < s.len() {
        // Find next '<'
        let rel = memchr(b'<', &s[p..])?;
        p += rel;

        // Need "<?"
        if p + 1 >= s.len() || s[p + 1] != b'?' {
            p += 1;
            continue;
        }

        // Third byte decides most cases
        let b2 = s.get(p + 2).copied().unwrap_or(0);

        // <?=
        if b2 == b'=' {
            return Some((p, TokenKind::EchoOpen, 3));
        }

        if b2 == b'p' {
            // <?php
            if p + 4 < s.len()
                && s[p + 2] == b'p'
                && s[p + 3] == b'h'
                && s[p + 4] == b'p'
            {
                return Some((p, TokenKind::PhpOpen, 5));
            }

            // <?phxt (must be before <?phx)
            if p + 5 < s.len()
                && s[p + 2] == b'p'
                && s[p + 3] == b'h'
                && s[p + 4] == b'x'
                && s[p + 5] == b't'
            {
                return Some((p, TokenKind::PhxtOpen, 6));
            }

            // <?phx
            if p + 4 < s.len()
                && s[p + 2] == b'p'
                && s[p + 3] == b'h'
                && s[p + 4] == b'x'
            {
                return Some((p, TokenKind::PhxOpen, 5));
            }
        }

        // Skip "<?"
        p += 2;
    }

    None
}

static IS_WS: [bool; 256] = {
    let mut t = [false; 256];
    t[b' ' as usize] = true;
    t[b'\t' as usize] = true;
    t[b'\r' as usize] = true;
    t[b'\n' as usize] = true;
    t[0x0C] = true;
    t
};

#[inline(always)]
fn is_ws(b: u8) -> bool {
    IS_WS[b as usize]
}

static IS_ID_START: [bool; 256] = {
    let mut t = [false; 256];
    let mut c = b'A';
    while c <= b'Z' {
        t[c as usize] = true;
        c += 1;
    }
    let mut c = b'a';
    while c <= b'z' {
        t[c as usize] = true;
        c += 1;
    }
    t[b'_' as usize] = true;
    // non-ASCII start
    let mut c = 0x80u8;
    loop {
        t[c as usize] = true;
        if c == 0xFF {
            break;
        }
        c = c.wrapping_add(1);
    }
    t
};

static IS_ID_CONT: [bool; 256] = {
    let mut t = IS_ID_START;
    let mut c = b'0';
    while c <= b'9' {
        t[c as usize] = true;
        c += 1;
    }
    t
};

#[inline(always)]
fn is_ident_start(b: u8) -> bool {
    IS_ID_START[b as usize]
}

#[inline]
fn is_dec_digit(b: u8) -> bool {
    (b'0'..=b'9').contains(&b)
}

#[inline(always)]
fn kw_of_bytes(s: &[u8]) -> Option<TokenKind> {
    // ASCII only
    if s.iter().any(|&c| c >= 0x80) {
        return None;
    }

    use phynix_core::token::TokenKind::*;
    match s.len() {
        2 => match s {
            b"as" => Some(KwAs),
            b"do" => Some(KwDo),
            b"fn" => Some(KwFn),
            b"if" => Some(KwIf),
            b"or" => Some(KwOr),
            _ => None,
        },
        3 => match s {
            b"and" => Some(KwAnd),
            b"die" => Some(KwDie),
            b"for" => Some(KwFor),
            b"new" => Some(KwNew),
            b"try" => Some(KwTry),
            b"use" => Some(KwUse),
            b"var" => Some(KwVar),
            b"xor" => Some(KwXor),
            _ => None,
        },
        4 => match s {
            b"case" => Some(KwCase),
            b"echo" => Some(KwEcho),
            b"else" => Some(KwElse),
            b"enum" => Some(KwEnum),
            b"eval" => Some(KwEval),
            b"exit" => Some(KwExit),
            b"from" => Some(KwFrom),
            b"goto" => Some(KwGoto),
            b"list" => Some(KwList),
            b"self" => Some(KwSelf),
            _ => None,
        },
        5 => match s {
            b"array" => Some(KwArray),
            b"break" => Some(KwBreak),
            b"catch" => Some(KwCatch),
            b"class" => Some(KwClass),
            b"clone" => Some(KwClone),
            b"const" => Some(KwConst),
            b"empty" => Some(KwEmpty),
            b"endif" => Some(KwEndIf),
            b"final" => Some(KwFinal),
            b"isset" => Some(KwIsset),
            b"match" => Some(KwMatch),
            b"print" => Some(KwPrint),
            b"throw" => Some(KwThrow),
            b"trait" => Some(KwTrait),
            b"unset" => Some(KwUnset),
            b"while" => Some(KwWhile),
            b"yield" => Some(KwYield),
            _ => None,
        },
        6 => match s {
            b"elseif" => Some(KwElseIf),
            b"endfor" => Some(KwEndFor),
            b"global" => Some(KwGlobal),
            b"parent" => Some(KwParent),
            b"public" => Some(KwPublic),
            b"return" => Some(KwReturn),
            b"static" => Some(KwStatic),
            b"switch" => Some(KwSwitch),
            _ => None,
        },
        7 => match s {
            b"declare" => Some(KwDeclare),
            b"default" => Some(KwDefault),
            b"extends" => Some(KwExtends),
            b"finally" => Some(KwFinally),
            b"foreach" => Some(KwForeach),
            b"include" => Some(KwInclude),
            b"private" => Some(KwPrivate),
            b"require" => Some(KwRequire),
            _ => None,
        },
        8 => match s {
            b"abstract" => Some(KwAbstract),
            b"callable" => Some(KwCallable),
            b"continue" => Some(KwContinue),
            b"endwhile" => Some(KwEndWhile),
            b"function" => Some(KwFunction),
            b"readonly" => Some(KwReadonly),
            _ => None,
        },
        9 => match s {
            b"endswitch" => Some(KwEndSwitch),
            b"insteadof" => Some(KwInsteadof),
            b"interface" => Some(KwInterface),
            b"namespace" => Some(KwNamespace),
            b"protected" => Some(KwProtected),
            _ => None,
        },
        10 => match s {
            b"endforeach" => Some(KwEndForeach),
            b"enddeclare" => Some(KwEndDeclare),
            b"implements" => Some(KwImplements),
            b"instanceof" => Some(KwInstanceof),
            _ => None,
        },
        12 => match s {
            b"include_once" => Some(KwIncludeOnce),
            b"require_once" => Some(KwRequireOnce),
            _ => None,
        },
        _ => None,
    }
}
