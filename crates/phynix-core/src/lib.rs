pub mod diagnostics;
pub mod token;

pub enum LanguageKind {
    /// .php
    PhpCompat,

    /// .phx
    PhxCode,

    /// .phxt
    PhxtTemplate,
}

pub enum Strictness {
    /// Legacy not allowed
    Strict,

    /// Legacy allowed but deprecated
    Lenient,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub const EMPTY: Span = Span::at(0);

    /// Creates a zero-width span at the given position.
    #[inline]
    pub const fn at(pos: u32) -> Span {
        Span {
            start: pos,
            end: pos,
        }
    }
}

pub trait Spanned {
    fn span(&self) -> Span;
}
