use crate::ast::{
    Arg, ArrayItemExpr, BinOpKind, Block, CastKind, ClassNameRef, ClosureUse,
    Ident, IncludeKind, ListItemExpr, MatchArm, Param, QualifiedName,
    StringStyle, TypeRef, UnOpKind,
};
use phynix_core::{Span, Spanned};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum Expr {
    /// 42, -7
    IntLiteral {
        value: i64,
        #[serde(skip)]
        span: Span,
    },

    /// 3.14, 2.0e10
    FloatLiteral {
        value: f64,
        #[serde(skip)]
        span: Span,
    },

    /// "string" or 'string'
    StringLiteral {
        style: StringStyle,
        #[serde(skip)]
        span: Span,
    },

    /// true / false
    BoolLiteral {
        value: bool,
        #[serde(skip)]
        span: Span,
    },

    /// null
    NullLiteral {
        #[serde(skip)]
        span: Span,
    },

    /// [expr, expr2 => expr3, ...]
    ArrayLiteral {
        items: Vec<ArrayItemExpr>,
        #[serde(skip)]
        span: Span,
    },

    /// $foo
    VarRef {
        name: Ident,
        #[serde(skip)]
        span: Span,
    },

    /// $$foo, ${$bar}
    VariableVariable {
        target: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// $arr[expr]
    ArrayIndex {
        array: Box<Expr>,
        index: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// $arr[] = expr
    ArrayAppend {
        array: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// $obj->prop
    PropertyFetch {
        target: Box<Expr>,
        property: Ident,
        #[serde(skip)]
        span: Span,
    },

    /// $obj?->prop
    NullsafePropertyFetch {
        target: Box<Expr>,
        property: Ident,
        #[serde(skip)]
        span: Span,
    },

    /// CONSTANT
    ConstFetch {
        name: QualifiedName,
        #[serde(skip)]
        span: Span,
    },

    /// ClassName::class
    NameRef {
        name: QualifiedName,
        #[serde(skip)]
        span: Span,
    },

    /// ClassName::CONSTANT
    ClassConstFetch {
        class_name: ClassNameRef,
        constant: Ident,
        #[serde(skip)]
        span: Span,
    },

    /// ClassName::$prop
    StaticPropertyFetch {
        class_name: ClassNameRef,
        property: Ident,
        #[serde(skip)]
        span: Span,
    },

    /// foo($a, $b) or $callable($a)
    FunctionCall {
        callee: Box<Expr>,
        args: Vec<Arg>,
        #[serde(skip)]
        span: Span,
    },

    /// $obj->method($a)
    MethodCall {
        target: Box<Expr>,
        method: Ident,
        args: Vec<Arg>,
        #[serde(skip)]
        span: Span,
    },

    /// $obj?->method($a)
    NullsafeMethodCall {
        target: Box<Expr>,
        method: Ident,
        args: Vec<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// ClassName::method($a)
    StaticCall {
        class: ClassNameRef,
        method: Ident,
        args: Vec<Arg>,
        #[serde(skip)]
        span: Span,
    },

    /// new ClassName($args)
    New {
        class: Box<Expr>,
        args: Vec<Arg>,
        #[serde(skip)]
        span: Span,
    },

    /// -$a
    UnaryOp {
        op: UnOpKind,
        expr: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// $a + $b
    BinaryOp {
        op: BinOpKind,
        left: Box<Expr>,
        right: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// $expr instanceof ClassName
    InstanceOf {
        expr: Box<Expr>,
        class: ClassNameRef,
        #[serde(skip)]
        span: Span,
    },

    /// $a ?? $b
    NullCoalesce {
        left: Box<Expr>,
        right: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// $cond ? $then : $else
    Ternary {
        condition: Box<Expr>,
        then_expr: Option<Box<Expr>>,
        else_expr: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// ($expr)
    Parenthesized {
        inner: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// cast: (int)$x
    Cast {
        kind: CastKind,
        expr: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// clone $obj
    Clone {
        expr: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// $a = 5
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// $a += 5
    CompoundAssign {
        op: BinOpKind,
        target: Box<Expr>,
        value: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// ??= / .= / += / etc.
    CoalesceAssign {
        target: Box<Expr>,
        value: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// match (...) { ... }
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
        #[serde(skip)]
        span: Span,
    },

    /// include / require / etc.
    Include {
        kind: IncludeKind,
        target: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// throw $expr
    Throw {
        expr: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    Exit {
        arg: Option<Box<Expr>>,
        #[serde(skip)]
        span: Span,
    },

    /// isset($a, $b, ...)
    Isset {
        exprs: Vec<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// empty($a)
    Empty {
        expr: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// yield $x / yield $k => $v
    Yield {
        key: Option<Box<Expr>>,
        value: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// yield from $iter
    YieldFrom {
        expr: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// function (...) use (...) { ... }
    Closure {
        is_static: bool,
        params: Vec<Param>,
        uses: Vec<ClosureUse>,
        return_type: Option<TypeRef>,
        body: Block,
        #[serde(skip)]
        span: Span,
    },

    /// fn (...) => expr
    ArrowClosure {
        is_static: bool,
        params: Vec<Param>,
        uses: Vec<ClosureUse>,
        return_type: Option<TypeRef>,
        body: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// ++$x
    PrefixInc {
        target: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// --$x
    PrefixDec {
        target: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// $x++
    PostfixInc {
        target: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// $x--
    PostfixDec {
        target: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// $obj->{$prop}
    DynamicPropertyFetch {
        target: Box<Expr>,
        property_expr: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// $obj->{$method}($args)
    DynamicMethodCall {
        target: Box<Expr>,
        method_expr: Box<Expr>,
        args: Vec<Arg>,
        #[serde(skip)]
        span: Span,
    },

    /// `ls -la`
    ShellExec {
        #[serde(skip)]
        span: Span,
    },

    /// list($a, $b) = ...
    ListDestructure {
        items: Vec<ListItemExpr>,
        #[serde(skip)]
        span: Span,
    },

    /// print $x
    Print {
        expr: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    /// eval($code)
    Eval {
        expr: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },

    Error {
        #[serde(skip)]
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
            | Expr::Eval { span, .. }
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
            | Expr::Print { span, .. }
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
