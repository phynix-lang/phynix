use crate::ast::{Block, Expr, Ident, Param, TypeRef};
use phynix_core::Span;

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

bitflags::bitflags! {
    #[derive(Debug)]
    pub struct ClassFlags: u8 {
        const ABSTRACT = 0b0000_0001;
        const FINAL    = 0b0000_0010;
        const READONLY = 0b0000_0100;
    }
}
