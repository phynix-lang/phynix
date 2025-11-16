use crate::ast::{ClassNameRef, Expr, Ident, QualifiedName, TypeRef};
use phynix_core::{Span, Spanned};
#[derive(Debug)]
pub enum Stmt {
    HtmlChunk {
        span: Span,
    },

    ExprStmt {
        expr: Expr,
        span: Span,
    },

    Assign {
        target: Ident,
        value: Expr,
        span: Span,
    },

    Echo {
        exprs: Vec<Expr>,
        span: Span,
    },

    Return {
        expr: Option<Expr>,
        span: Span,
    },

    Throw {
        expr: Expr,
        span: Span,
    },

    New {
        class: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },

    Noop {
        span: Span,
    },

    VarDecl {
        name: Ident,
        type_annotation: Option<TypeRef>,
        init: Option<Expr>,
        span: Span,
    },

    ConstDecl {
        name: Ident,
        value: Option<Expr>,
        span: Span,
    },

    Function {
        name: Ident,
        params: Vec<Param>,
        return_type: Option<TypeRef>,
        body: Block,
        span: Span,
    },

    Class {
        flags: ClassFlags,
        name: ClassNameRef,
        extends: Option<QualifiedName>,
        implements: Vec<QualifiedName>,
        body: Vec<ClassMember>,
        span: Span,
    },

    Interface {
        name: Ident,
        extends: Vec<QualifiedName>,
        body: Vec<ClassMember>,
        span: Span,
    },

    Trait {
        name: Ident,
        body: Vec<ClassMember>,
        span: Span,
    },

    Enum {
        name: Ident,
        backed_type: Option<TypeRef>,
        implements: Vec<QualifiedName>,
        body: Vec<ClassMember>,
        span: Span,
    },

    If {
        cond: Expr,
        then_block: Block,
        else_if_blocks: Vec<(Expr, Block)>,
        else_block: Option<Block>,
        span: Span,
    },

    Switch {
        cond: Expr,
        cases: Vec<SwitchCase>,
        span: Span,
    },

    While {
        cond: Option<Expr>,
        body: Block,
        span: Span,
    },

    DoWhile {
        body: Block,
        cond: Expr,
        span: Span,
    },

    For {
        init: Option<Expr>,
        cond: Option<Expr>,
        increment: Option<Expr>,
        body: Block,
        span: Span,
    },

    Foreach {
        expr: Option<Expr>,
        key: Option<Expr>,
        value: Option<Expr>,
        body: Block,
        span: Span,
    },

    Break {
        level: Option<Expr>,
        span: Span,
    },

    Continue {
        level: Option<Expr>,
        span: Span,
    },

    Goto {
        target: Ident,
        span: Span,
    },

    Label {
        name: Ident,
        span: Span,
    },

    Try {
        try_block: Block,
        catches: Vec<CatchClause>,
        finally_block: Option<Block>,
        span: Span,
    },

    Namespace {
        name: Option<QualifiedName>,
        body: Block,
        span: Span,
    },

    Use {
        imports: Vec<UseImport>,
        span: Span,
    },

    Declare {
        strict_types: Option<bool>,
        span: Span,
    },

    Global {
        span: Span,
    },
}

#[derive(Debug)]
pub struct Block {
    pub items: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug)]
pub struct Param {
    pub name: Ident,
    pub type_annotation: Option<TypeRef>,
    pub default: Option<Expr>,
    pub span: Span,
}

impl Spanned for Stmt {
    fn span(&self) -> Span {
        match self {
            Stmt::Assign { span, .. }
            | Stmt::Break { span, .. }
            | Stmt::Class { span, .. }
            | Stmt::ConstDecl { span, .. }
            | Stmt::Continue { span, .. }
            | Stmt::Declare { span, .. }
            | Stmt::DoWhile { span, .. }
            | Stmt::Echo { span, .. }
            | Stmt::Enum { span, .. }
            | Stmt::ExprStmt { span, .. }
            | Stmt::For { span, .. }
            | Stmt::Foreach { span, .. }
            | Stmt::Function { span, .. }
            | Stmt::Global { span, .. }
            | Stmt::Goto { span, .. }
            | Stmt::HtmlChunk { span, .. }
            | Stmt::If { span, .. }
            | Stmt::Interface { span, .. }
            | Stmt::Label { span, .. }
            | Stmt::Namespace { span, .. }
            | Stmt::New { span, .. }
            | Stmt::Noop { span, .. }
            | Stmt::Return { span, .. }
            | Stmt::Switch { span, .. }
            | Stmt::Throw { span, .. }
            | Stmt::Trait { span, .. }
            | Stmt::Try { span, .. }
            | Stmt::Use { span, .. }
            | Stmt::VarDecl { span, .. }
            | Stmt::While { span, .. } => *span,
        }
    }
}

#[derive(Debug)]
pub enum ClassMember {
    Property {
        name: Ident,
        type_annotation: Option<TypeRef>,
        default: Option<Expr>,
        span: Span,
    },

    Method {
        name: Ident,
        params: Vec<Param>,
        return_type: Option<TypeRef>,
        body: Option<Block>,
        span: Span,
    },

    Const {
        name: Ident,
        value: Expr,
        span: Span,
    },
}

#[derive(Debug, Copy, Clone)]
pub enum UseKind {
    Normal,
    Function,
    Const,
}

#[derive(Debug)]
pub struct UseImport {
    pub kind: UseKind,
    pub full_name: QualifiedName,
    pub alias: Option<Ident>,
    pub span: Span,
}

bitflags::bitflags! {
    #[derive(Debug)]
    pub struct ClassFlags: u8 {
        const ABSTRACT = 0b0000_0001;
        const FINAL    = 0b0000_0010;
        const READONLY = 0b0000_0100;
    }
}

#[derive(Debug)]
pub struct CatchClause {
    pub exception_types: Vec<TypeRef>,
    pub var: Option<Ident>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug)]
pub struct SwitchCase {
    pub condition: Option<Expr>,
    pub body: Block,
    pub span: Span,
}
