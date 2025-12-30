use crate::ast::QualifiedName;
use phynix_core::{Span, Spanned};
use serde::Serialize;

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub enum TypeRef {
    Named {
        name: QualifiedName,
        #[serde(skip)]
        span: Span,
    },

    Nullable {
        inner: Box<TypeRef>,
        #[serde(skip)]
        span: Span,
    },

    Union {
        types: Vec<TypeRef>,
        #[serde(skip)]
        span: Span,
    },

    Intersection {
        types: Vec<TypeRef>,
        #[serde(skip)]
        span: Span,
    },

    Generic {
        base: Box<TypeRef>,
        args: Vec<TypeRef>,
        #[serde(skip)]
        span: Span,
    },

    ArrayOf {
        element: Box<TypeRef>,
        #[serde(skip)]
        span: Span,
    },

    Callable {
        params: Vec<TypeRef>,
        return_type: Option<Box<TypeRef>>,
        #[serde(skip)]
        span: Span,
    },

    Keyword {
        kind: BuiltInType,
        #[serde(skip)]
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
