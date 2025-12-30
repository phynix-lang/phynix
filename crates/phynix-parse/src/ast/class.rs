use crate::ast::{Block, Expr, Ident, Param, TypeRef};
use phynix_core::Span;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum ClassMember {
    Property {
        name: Ident,
        type_annotation: Option<TypeRef>,
        default: Option<Expr>,
        #[serde(skip)]
        span: Span,
    },

    Method {
        name: Ident,
        params: Vec<Param>,
        return_type: Option<TypeRef>,
        body: Option<Block>,
        #[serde(skip)]
        span: Span,
    },

    Const {
        name: Ident,
        value: Expr,
        #[serde(skip)]
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

impl Serialize for ClassFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(self.bits())
    }
}
