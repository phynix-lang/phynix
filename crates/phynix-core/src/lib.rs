use serde::Serialize;

pub mod diagnostics;
pub mod token;

#[derive(Debug, Clone, Serialize)]
pub enum LanguageKind {
    /// .php
    Php,

    /// .phx
    PhxCode,

    /// .phxt
    PhxTemplate,
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize,
)]
pub enum PhpVersion {
    Php70,
    Php71,
    Php72,
    Php73,
    Php74,
    Php80,
    Php81,
    Php82,
    Php83,
    Php84,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PhynixConfig {
    pub language: LanguageKind,
    pub target_php_version: PhpVersion,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
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
