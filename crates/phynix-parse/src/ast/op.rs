use serde::Serialize;

#[derive(Copy, Clone, Debug, Serialize)]
pub enum BinOpKind {
    /// Arithmetic
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Mod, // %
    Pow, // **

    /// String concatenation
    Concat, // .

    /// Logical
    AndAnd, // &&
    OrOr,         // ||
    NullCoalesce, // ??
    Or,           // or
    Xor,          // xor
    And,          // and

    /// Bitwise / Shifts
    BitAnd, // &
    BitOr,  // |
    BitXor, // ^
    Shl,    // <<
    Shr,    // >>

    /// Comparison
    CmpEqStrict, // ===
    CmpNeStrict, // !==
    CmpEq,       // ==
    CmpNe,       // !=

    /// Ordering
    CmpLt, // <
    CmpLe, // <=
    CmpGt, // >
    CmpGe, // >=

    /// Spaceship
    CmpSpaceship, // <=>
}

#[derive(Debug, Serialize)]
pub enum UnOpKind {
    Neg,      // -$x
    Not,      // !$x
    BitNot,   // ~$x
    Suppress, // @$x
    Ref,      // &$x
    Plus,     // +$x
}

#[derive(Debug, Serialize)]
pub enum CastKind {
    Int,
    Float,
    String,
    Array,
    Object,
    Bool,
    Unset,
}

#[derive(Debug, Serialize)]
pub enum IncludeKind {
    Include,
    IncludeOnce,
    Require,
    RequireOnce,
}
