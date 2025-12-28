use crate::Span;

#[derive(Copy, Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenKind {
    HtmlChunk,

    /// `<?phxt`
    PhxtOpen,
    /// `<?phx`
    PhxOpen,
    /// `<?php`
    PhpOpen,
    /// `<?=`
    EchoOpen,
    /// `?>`
    PhpClose,

    /// Whitespace
    WS,

    /// `#[`
    AttrOpen,

    /// `#`
    HashComment,
    /// `//`
    LineComment,
    /// `/** */`
    Docblock,
    /// `/* */`
    BlockComment,

    /// `$identifier`
    VarIdent,
    /// `identifier`
    Ident,

    /// `$`
    Dollar,

    /// `\`
    Backslash,

    /// `12.34`, `12e5`
    Float,

    /// `0`, `123`, `0xff`, `0b1010`
    Int,

    /// `"string"`
    StrDq,
    /// `'string'`
    StrSq,
    /// `` `string` ``
    StrBt,

    /// `?->`
    NullsafeArrow,

    /// `>>=`
    ShrEq,
    /// `<<=`
    ShlEq,
    /// `**=`
    PowEq,
    /// `??=`
    NullCoalesceAssign,
    /// `.=`
    DotEq,
    /// `+=`
    PlusEq,
    /// `-=`
    MinusEq,
    /// `*=`
    MulEq,
    /// `/=`
    DivEq,
    /// `%=`
    ModEq,
    /// `&=`
    AmpEq,
    /// `|=`
    PipeEq,
    /// `^=`
    CaretEq,

    /// `===`
    StrictEq,
    /// `!==`
    StrictNe,
    /// `<=>`
    Spaceship,
    /// `??`
    NullCoalesce,
    /// `&&`
    AndAnd,
    /// `||`
    OrOr,
    /// `<<`
    Shl,
    /// `>>`
    Shr,
    /// `**`
    Pow,
    /// `::`
    ColCol,
    /// `->`
    Arrow,
    /// `=>`
    FatArrow,
    /// `==`
    EqEq,
    /// `!=`
    NotEq,
    /// `<=`
    Le,
    /// `>=`
    Ge,
    /// `...`
    Ellipsis,

    /// `++`
    PlusPlus,
    /// `--`
    MinusMinus,

    /// `=`
    Eq,
    /// `<`
    Lt,
    /// `>`
    Gt,
    /// `?`
    Question,
    /// `.`
    Dot,

    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `&`
    Amp,
    /// `|`
    Pipe,
    /// `^`
    Caret,
    /// `@`
    Silence,
    /// `!`
    Bang,
    /// `~`
    Tilde,

    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `[`
    LBracket,
    /// `]`
    RBracket,

    /// `;`
    Semicolon,
    /// `,`
    Comma,
    /// `:`
    Colon,

    KwAbstract,
    KwAnd,
    KwArray,
    KwAs,
    KwBreak,
    KwCallable,
    KwCase,
    KwCatch,
    KwClass,
    KwClone,
    KwConst,
    KwContinue,
    KwDeclare,
    KwDefault,
    KwDie,
    KwDo,
    KwEcho,
    KwElse,
    KwElseIf,
    KwEmpty,
    KwEndDeclare,
    KwEndFor,
    KwEndForeach,
    KwEndIf,
    KwEndSwitch,
    KwEndWhile,
    KwEnum,
    KwEval,
    KwExit,
    KwExtends,
    KwFinal,
    KwFinally,
    KwFn,
    KwFor,
    KwForeach,
    KwFrom,
    KwFunction,
    KwGlobal,
    KwGoto,
    KwIf,
    KwImplements,
    KwInclude,
    /// `include_once`
    KwIncludeOnce,
    KwInstanceof,
    KwInsteadof,
    KwInterface,
    KwIsset,
    KwList,
    KwMatch,
    KwNamespace,
    KwNew,
    KwOr,
    KwParent,
    KwPrint,
    KwPrivate,
    KwProtected,
    KwPublic,
    KwReadonly,
    KwRequire,
    /// `require_once`
    KwRequireOnce,
    KwReturn,
    KwSelf,
    KwStatic,
    KwSwitch,
    KwThrow,
    KwTrait,
    KwTry,
    KwUnset,
    KwUse,
    KwVar,
    KwWhile,
    KwXor,
    KwYield,

    Eof,
}

impl TokenKind {
    /// Returns a human-readable representation
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::HtmlChunk => "HTML",
            Self::PhxtOpen => "<?phxt",
            Self::PhxOpen => "<?phx",
            Self::PhpOpen => "<?php",
            Self::EchoOpen => "<?=",
            Self::PhpClose => "?>",
            Self::WS => "whitespace",
            Self::AttrOpen => "#[",
            Self::HashComment | Self::LineComment => "comment",
            Self::Docblock => "docblock",
            Self::BlockComment => "comment",
            Self::VarIdent => "variable",
            Self::Ident => "identifier",
            Self::Dollar => "$",
            Self::Backslash => "\\",
            Self::Float => "float literal",
            Self::Int => "integer literal",
            Self::StrDq | Self::StrSq => "string",
            Self::StrBt => "shell command",
            Self::NullsafeArrow => "?->",
            Self::ShrEq => ">>=",
            Self::ShlEq => "<<=",
            Self::PowEq => "**=",
            Self::NullCoalesceAssign => "??=",
            Self::DotEq => ".=",
            Self::PlusEq => "+=",
            Self::MinusEq => "-=",
            Self::MulEq => "*=",
            Self::DivEq => "/=",
            Self::ModEq => "%=",
            Self::AmpEq => "&=",
            Self::PipeEq => "|=",
            Self::CaretEq => "^=",
            Self::StrictEq => "===",
            Self::StrictNe => "!==",
            Self::Spaceship => "<=>",
            Self::NullCoalesce => "??",
            Self::AndAnd => "&&",
            Self::OrOr => "||",
            Self::Shl => "<<",
            Self::Shr => ">>",
            Self::Pow => "**",
            Self::ColCol => "::",
            Self::Arrow => "->",
            Self::FatArrow => "=>",
            Self::EqEq => "==",
            Self::NotEq => "!=",
            Self::Le => "<=",
            Self::Ge => ">=",
            Self::Ellipsis => "...",
            Self::PlusPlus => "++",
            Self::MinusMinus => "--",
            Self::Eq => "=",
            Self::Lt => "<",
            Self::Gt => ">",
            Self::Question => "?",
            Self::Dot => ".",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Star => "*",
            Self::Slash => "/",
            Self::Percent => "%",
            Self::Amp => "&",
            Self::Pipe => "|",
            Self::Caret => "^",
            Self::Silence => "@",
            Self::Bang => "!",
            Self::Tilde => "~",
            Self::LBrace => "{",
            Self::RBrace => "}",
            Self::LParen => "(",
            Self::RParen => ")",
            Self::LBracket => "[",
            Self::RBracket => "]",
            Self::Semicolon => ";",
            Self::Comma => ",",
            Self::Colon => ":",
            Self::KwAbstract => "abstract",
            Self::KwAnd => "and",
            Self::KwArray => "array",
            Self::KwAs => "as",
            Self::KwBreak => "break",
            Self::KwCallable => "callable",
            Self::KwCase => "case",
            Self::KwCatch => "catch",
            Self::KwClass => "class",
            Self::KwClone => "clone",
            Self::KwConst => "const",
            Self::KwContinue => "continue",
            Self::KwDeclare => "declare",
            Self::KwDefault => "default",
            Self::KwDie => "die",
            Self::KwDo => "do",
            Self::KwEcho => "echo",
            Self::KwElse => "else",
            Self::KwElseIf => "elseif",
            Self::KwEmpty => "empty",
            Self::KwEndDeclare => "enddeclare",
            Self::KwEndFor => "endfor",
            Self::KwEndForeach => "endforeach",
            Self::KwEndIf => "endif",
            Self::KwEndSwitch => "endswitch",
            Self::KwEndWhile => "endwhile",
            Self::KwEnum => "enum",
            Self::KwEval => "eval",
            Self::KwExit => "exit",
            Self::KwExtends => "extends",
            Self::KwFinal => "final",
            Self::KwFinally => "finally",
            Self::KwFn => "fn",
            Self::KwFor => "for",
            Self::KwForeach => "foreach",
            Self::KwFrom => "from",
            Self::KwFunction => "function",
            Self::KwGlobal => "global",
            Self::KwGoto => "goto",
            Self::KwIf => "if",
            Self::KwImplements => "implements",
            Self::KwInclude => "include",
            Self::KwIncludeOnce => "include_once",
            Self::KwInstanceof => "instanceof",
            Self::KwInsteadof => "insteadof",
            Self::KwInterface => "interface",
            Self::KwIsset => "isset",
            Self::KwList => "list",
            Self::KwMatch => "match",
            Self::KwNamespace => "namespace",
            Self::KwNew => "new",
            Self::KwOr => "or",
            Self::KwParent => "parent",
            Self::KwPrint => "print",
            Self::KwPrivate => "private",
            Self::KwProtected => "protected",
            Self::KwPublic => "public",
            Self::KwReadonly => "readonly",
            Self::KwRequire => "require",
            Self::KwRequireOnce => "require_once",
            Self::KwReturn => "return",
            Self::KwSelf => "self",
            Self::KwStatic => "static",
            Self::KwSwitch => "switch",
            Self::KwThrow => "throw",
            Self::KwTrait => "trait",
            Self::KwTry => "try",
            Self::KwUnset => "unset",
            Self::KwUse => "use",
            Self::KwVar => "var",
            Self::KwWhile => "while",
            Self::KwXor => "xor",
            Self::KwYield => "yield",
            Self::Eof => "end of file",
        }
    }

    #[inline]
    pub fn is_trivia(&self) -> bool {
        matches!(
            self,
            TokenKind::WS
                | TokenKind::HashComment
                | TokenKind::LineComment
                | TokenKind::BlockComment
                | TokenKind::Docblock
        )
    }
}
