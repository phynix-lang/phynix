pub mod diagnostics;

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

pub trait Spanned {
    fn span(&self) -> Span;
}
