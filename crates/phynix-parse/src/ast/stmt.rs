use crate::ast::{
    AttributeGroup, CatchClause, ClassFlags, ClassMember, ClassNameRef, Expr,
    Ident, Param, QualifiedName, SwitchCase, TypeRef, UseImport,
};
use phynix_core::{Span, Spanned};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum Stmt {
    HtmlChunk {
        #[serde(skip)]
        span: Span,
    },

    ExprStmt {
        expr: Expr,
        #[serde(skip)]
        span: Span,
    },

    Assign {
        target: Ident,
        value: Expr,
        #[serde(skip)]
        span: Span,
    },

    Echo {
        exprs: Vec<Expr>,
        #[serde(skip)]
        span: Span,
    },

    Return {
        expr: Option<Expr>,
        #[serde(skip)]
        span: Span,
    },

    Throw {
        expr: Expr,
        #[serde(skip)]
        span: Span,
    },

    New {
        class: Box<Expr>,
        args: Vec<Expr>,
        #[serde(skip)]
        span: Span,
    },

    Noop {
        #[serde(skip)]
        span: Span,
    },

    VarDecl {
        name: Ident,
        type_annotation: Option<TypeRef>,
        init: Option<Expr>,
        #[serde(skip)]
        span: Span,
    },

    ConstDecl {
        name: Ident,
        value: Option<Expr>,
        #[serde(skip)]
        span: Span,
    },

    Function {
        name: Ident,
        params: Vec<Param>,
        return_type: Option<TypeRef>,
        body: Block,
        #[serde(skip)]
        span: Span,
    },

    Class {
        flags: ClassFlags,
        name: ClassNameRef,
        extends: Option<QualifiedName>,
        implements: Vec<QualifiedName>,
        body: Vec<ClassMember>,
        attributes: Vec<AttributeGroup>,
        #[serde(skip)]
        span: Span,
    },

    Interface {
        name: Ident,
        extends: Vec<QualifiedName>,
        body: Vec<ClassMember>,
        #[serde(skip)]
        span: Span,
    },

    Trait {
        name: Ident,
        body: Vec<ClassMember>,
        #[serde(skip)]
        span: Span,
    },

    Enum {
        name: Ident,
        backed_type: Option<TypeRef>,
        implements: Vec<QualifiedName>,
        body: Vec<ClassMember>,
        #[serde(skip)]
        span: Span,
    },

    If {
        cond: Expr,
        then_block: Block,
        else_if_blocks: Vec<(Expr, Block)>,
        else_block: Option<Block>,
        #[serde(skip)]
        span: Span,
    },

    Switch {
        cond: Expr,
        cases: Vec<SwitchCase>,
        #[serde(skip)]
        span: Span,
    },

    While {
        cond: Option<Expr>,
        body: Block,
        #[serde(skip)]
        span: Span,
    },

    DoWhile {
        body: Block,
        cond: Expr,
        #[serde(skip)]
        span: Span,
    },

    For {
        init: Option<Expr>,
        cond: Option<Expr>,
        increment: Option<Expr>,
        body: Block,
        #[serde(skip)]
        span: Span,
    },

    Foreach {
        expr: Option<Expr>,
        key: Option<Expr>,
        value: Option<Expr>,
        body: Block,
        #[serde(skip)]
        span: Span,
    },

    Break {
        level: Option<Expr>,
        #[serde(skip)]
        span: Span,
    },

    Continue {
        level: Option<Expr>,
        #[serde(skip)]
        span: Span,
    },

    Goto {
        target: Ident,
        #[serde(skip)]
        span: Span,
    },

    Label {
        name: Ident,
        #[serde(skip)]
        span: Span,
    },

    Try {
        try_block: Block,
        catches: Vec<CatchClause>,
        finally_block: Option<Block>,
        #[serde(skip)]
        span: Span,
    },

    Namespace {
        name: Option<QualifiedName>,
        body: Block,
        #[serde(skip)]
        span: Span,
    },

    Use {
        imports: Vec<UseImport>,
        #[serde(skip)]
        span: Span,
    },

    Declare {
        strict_types: Option<bool>,
        #[serde(skip)]
        span: Span,
    },

    Global {
        #[serde(skip)]
        span: Span,
    },

    Unset {
        exprs: Vec<Expr>,
        #[serde(skip)]
        span: Span,
    },
}

#[derive(Debug, Serialize)]
pub struct Block {
    pub items: Vec<Stmt>,
    #[serde(skip)]
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
            | Stmt::Unset { span, .. }
            | Stmt::Use { span, .. }
            | Stmt::VarDecl { span, .. }
            | Stmt::While { span, .. } => *span,
        }
    }
}
