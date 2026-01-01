use crate::ast::{Block, Expr, Ident, Param, QualifiedName, TypeRef};
use phynix_core::Span;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum ClassMember {
    TraitUse {
        traits: Vec<QualifiedName>,
        #[serde(skip)]
        span: Span,
    },

    Property {
        name: Ident,
        flags: MemberFlags,
        type_annotation: Option<TypeRef>,
        default: Option<Expr>,
        #[serde(skip)]
        span: Span,
    },

    Method {
        name: Ident,
        flags: MemberFlags,
        params: Vec<Param>,
        return_type: Option<TypeRef>,
        body: Option<Block>,
        #[serde(skip)]
        span: Span,
    },

    Const {
        name: Ident,
        flags: MemberFlags,
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

    #[derive(Debug)]
    pub struct MemberFlags: u16 {
        const PUBLIC    = 0b0000_0000_0001;
        const PROTECTED = 0b0000_0000_0010;
        const PRIVATE   = 0b0000_0000_0100;
        const STATIC    = 0b0000_0000_1000;
        const ABSTRACT  = 0b0000_0001_0000;
        const FINAL     = 0b0000_0010_0000;
        const READONLY  = 0b0000_0100_0000;
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

impl Serialize for MemberFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u16(self.bits())
    }
}
