use crate::ast::QualifiedName;
use phynix_core::{Span, Spanned};
#[derive(Debug)]
pub enum BuiltInType {
    Int,
    String,
    Bool,
    Array,
    Object,
    Callable,
    Mixed,
    Never,
    Void,
    Null,
    False,
    True,
}

#[derive(Debug)]
pub enum TypeRef {
    Named {
        name: QualifiedName,
        span: Span,
    },

    Nullable {
        inner: Box<TypeRef>,
        span: Span,
    },

    Union {
        types: Vec<TypeRef>,
        span: Span,
    },

    Intersection {
        types: Vec<TypeRef>,
        span: Span,
    },

    Generic {
        base: Box<TypeRef>,
        args: Vec<TypeRef>,
        span: Span,
    },

    ArrayOf {
        element: Box<TypeRef>,
        span: Span,
    },

    Callable {
        params: Vec<TypeRef>,
        return_type: Option<Box<TypeRef>>,
        span: Span,
    },

    Keyword {
        kind: BuiltInType,
        span: Span,
    },
}

impl Spanned for TypeRef {
    fn span(&self) -> Span {
        match self {
            TypeRef::Named { span, .. }
            | TypeRef::Nullable { span, .. }
            | TypeRef::Union { span, .. }
            | TypeRef::Intersection { span, .. }
            | TypeRef::Generic { span, .. }
            | TypeRef::ArrayOf { span, .. }
            | TypeRef::Callable { span, .. }
            | TypeRef::Keyword { span, .. } => *span,
        }
    }
}
