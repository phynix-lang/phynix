use crate::ast::{
    Block, ClassNameRef, Ident, Param, QualifiedName, StringStyle, TypeRef,
};
use phynix_core::{Span, Spanned};
#[derive(Debug)]
pub enum Expr {
    /// 42, -7
    IntLiteral {
        value: i64,
        span: Span,
    },

    /// 3.14, 2.0e10
    FloatLiteral {
        value: f64,
        span: Span,
    },

    /// "string" or 'string'
    StringLiteral {
        style: StringStyle,
        span: Span,
    },

    /// true / false
    BoolLiteral {
        value: bool,
        span: Span,
    },

    /// null
    NullLiteral {
        span: Span,
    },

    /// [expr, expr2 => expr3, ...]
    ArrayLiteral {
        items: Vec<ArrayItemExpr>,
        span: Span,
    },

    /// $foo
    VarRef {
        name: Ident,
        span: Span,
    },

    /// $$foo, ${$bar}
    VariableVariable {
        target: Box<Expr>,
        span: Span,
    },

    /// $arr[expr]
    ArrayIndex {
        array: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },

    /// $arr[] = expr
    ArrayAppend {
        array: Box<Expr>,
        span: Span,
    },

    /// $obj->prop
    PropertyFetch {
        target: Box<Expr>,
        property: Ident,
        span: Span,
    },

    /// $obj?->prop
    NullsafePropertyFetch {
        target: Box<Expr>,
        property: Ident,
        span: Span,
    },

    /// CONSTANT
    ConstFetch {
        name: QualifiedName,
        span: Span,
    },

    /// ClassName::class
    NameRef {
        name: QualifiedName,
        span: Span,
    },

    /// ClassName::CONSTANT
    ClassConstFetch {
        class_name: ClassNameRef,
        constant: Ident,
        span: Span,
    },

    /// ClassName::$prop
    StaticPropertyFetch {
        class_name: ClassNameRef,
        property: Ident,
        span: Span,
    },

    /// foo($a, $b) or $callable($a)
    FunctionCall {
        callee: Box<Expr>,
        args: Vec<Arg>,
        span: Span,
    },

    /// $obj->method($a)
    MethodCall {
        target: Box<Expr>,
        method: Ident,
        args: Vec<Arg>,
        span: Span,
    },

    /// $obj?->method($a)
    NullsafeMethodCall {
        target: Box<Expr>,
        method: Ident,
        args: Vec<Expr>,
        span: Span,
    },

    /// ClassName::method($a)
    StaticCall {
        class: ClassNameRef,
        method: Ident,
        args: Vec<Arg>,
        span: Span,
    },

    /// new ClassName($args)
    New {
        class: Box<Expr>,
        args: Vec<Arg>,
        span: Span,
    },

    /// -$a
    UnaryOp {
        op: UnOpKind,
        expr: Box<Expr>,
        span: Span,
    },

    /// $a + $b
    BinaryOp {
        op: BinOpKind,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },

    /// $expr instanceof ClassName
    InstanceOf {
        expr: Box<Expr>,
        class: QualifiedName,
        span: Span,
    },

    /// $a ?? $b
    NullCoalesce {
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },

    /// $cond ? $then : $else
    Ternary {
        condition: Box<Expr>,
        then_expr: Option<Box<Expr>>,
        else_expr: Box<Expr>,
        span: Span,
    },

    /// ($expr)
    Parenthesized {
        inner: Box<Expr>,
        span: Span,
    },

    /// cast: (int)$x
    Cast {
        kind: CastKind,
        expr: Box<Expr>,
        span: Span,
    },

    /// clone $obj
    Clone {
        expr: Box<Expr>,
        span: Span,
    },

    /// $a = 5
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },

    /// $a += 5
    CompoundAssign {
        op: BinOpKind,
        target: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },

    /// ??= / .= / += / etc.
    CoalesceAssign {
        target: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },

    /// match (...) { ... }
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },

    /// include / require / etc.
    Include {
        kind: IncludeKind,
        target: Box<Expr>,
        span: Span,
    },

    /// throw $expr
    Throw {
        expr: Box<Expr>,
        span: Span,
    },

    Exit {
        arg: Option<Box<Expr>>,
        span: Span,
    },

    /// isset($a, $b, ...)
    Isset {
        exprs: Vec<Expr>,
        span: Span,
    },

    /// empty($a)
    Empty {
        expr: Box<Expr>,
        span: Span,
    },

    /// yield $x / yield $k => $v
    Yield {
        key: Option<Box<Expr>>,
        value: Box<Expr>,
        span: Span,
    },

    /// yield from $iter
    YieldFrom {
        expr: Box<Expr>,
        span: Span,
    },

    /// function (...) use (...) { ... }
    Closure {
        is_static: bool,
        params: Vec<Param>,
        uses: Vec<ClosureUse>,
        return_type: Option<TypeRef>,
        body: Block,
        span: Span,
    },

    /// fn (...) => expr
    ArrowClosure {
        is_static: bool,
        params: Vec<Param>,
        uses: Vec<ClosureUse>,
        return_type: Option<TypeRef>,
        body: Box<Expr>,
        span: Span,
    },

    /// ++$x
    PrefixInc {
        target: Box<Expr>,
        span: Span,
    },

    /// --$x
    PrefixDec {
        target: Box<Expr>,
        span: Span,
    },

    /// $x++
    PostfixInc {
        target: Box<Expr>,
        span: Span,
    },

    /// $x--
    PostfixDec {
        target: Box<Expr>,
        span: Span,
    },

    /// $obj->{$prop}
    DynamicPropertyFetch {
        target: Box<Expr>,
        property_expr: Box<Expr>,
        span: Span,
    },

    /// $obj->{$method}($args)
    DynamicMethodCall {
        target: Box<Expr>,
        method_expr: Box<Expr>,
        args: Vec<Arg>,
        span: Span,
    },

    /// `ls -la`
    ShellExec {
        span: Span,
    },

    /// list($a, $b) = ...
    ListDestructure {
        items: Vec<ListItemExpr>,
        span: Span,
    },

    Error {
        span: Span,
    },
}

impl Spanned for Expr {
    fn span(&self) -> Span {
        match self {
            Expr::ArrayAppend { span, .. }
            | Expr::ArrayIndex { span, .. }
            | Expr::ArrayLiteral { span, .. }
            | Expr::ArrowClosure { span, .. }
            | Expr::Assign { span, .. }
            | Expr::BinaryOp { span, .. }
            | Expr::BoolLiteral { span, .. }
            | Expr::Cast { span, .. }
            | Expr::ClassConstFetch { span, .. }
            | Expr::Clone { span, .. }
            | Expr::Closure { span, .. }
            | Expr::CoalesceAssign { span, .. }
            | Expr::CompoundAssign { span, .. }
            | Expr::ConstFetch { span, .. }
            | Expr::DynamicMethodCall { span, .. }
            | Expr::DynamicPropertyFetch { span, .. }
            | Expr::Empty { span, .. }
            | Expr::Error { span, .. }
            | Expr::Exit { span, .. }
            | Expr::FloatLiteral { span, .. }
            | Expr::FunctionCall { span, .. }
            | Expr::Include { span, .. }
            | Expr::InstanceOf { span, .. }
            | Expr::IntLiteral { span, .. }
            | Expr::Isset { span, .. }
            | Expr::ListDestructure { span, .. }
            | Expr::Match { span, .. }
            | Expr::MethodCall { span, .. }
            | Expr::NameRef { span, .. }
            | Expr::New { span, .. }
            | Expr::NullCoalesce { span, .. }
            | Expr::NullLiteral { span, .. }
            | Expr::NullsafeMethodCall { span, .. }
            | Expr::NullsafePropertyFetch { span, .. }
            | Expr::Parenthesized { span, .. }
            | Expr::PostfixDec { span, .. }
            | Expr::PostfixInc { span, .. }
            | Expr::PrefixDec { span, .. }
            | Expr::PrefixInc { span, .. }
            | Expr::PropertyFetch { span, .. }
            | Expr::ShellExec { span, .. }
            | Expr::StaticCall { span, .. }
            | Expr::StaticPropertyFetch { span, .. }
            | Expr::StringLiteral { span, .. }
            | Expr::Ternary { span, .. }
            | Expr::Throw { span, .. }
            | Expr::UnaryOp { span, .. }
            | Expr::VariableVariable { span, .. }
            | Expr::VarRef { span, .. }
            | Expr::Yield { span, .. }
            | Expr::YieldFrom { span, .. } => *span,
        }
    }
}

#[derive(Copy, Clone, Debug)]
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

#[derive(Debug)]
pub enum UnOpKind {
    Neg,      // -$x
    Not,      // !$x
    BitNot,   // ~$x
    Suppress, // @$x
    Ref,      // &$x
    Plus,     // +$x
}

#[derive(Debug)]
pub enum CastKind {
    Int,
    Float,
    String,
    Array,
    Object,
    Bool,
    Unset,
}

#[derive(Debug)]
pub enum IncludeKind {
    Include,
    IncludeOnce,
    Require,
    RequireOnce,
}

#[derive(Debug)]
pub struct ArrayItemExpr {
    pub key: Option<Expr>,
    pub value: Expr,
    pub unpack: bool,
    pub span: Span,
}

#[derive(Debug)]
pub struct ClosureUse {
    pub by_ref: bool,
    pub name: Ident,
    pub span: Span,
}

#[derive(Debug)]
pub struct MatchArm {
    pub patterns: Vec<MatchPattern>,
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug)]
pub enum MatchPattern {
    Default { span: Span },
    Expr(Expr),
}

#[derive(Debug)]
pub struct Arg {
    pub name: Option<Ident>,
    pub unpack: bool,
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug)]
pub struct ListItemExpr {
    pub key: Option<Expr>,
    pub value: Option<Expr>,
    pub span: Span,
}
