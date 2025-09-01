//! TODO: write docs

use super::interner::{AdtId, Interner, Substitution, Type};
use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GenericArgType {
    Type,
}

#[derive(Debug)]
pub struct AdtData {
    pub name: Box<str>,
    pub generic_args: Box<[GenericArgType]>,
    // TODO: add bounds / where clauses
}

#[derive(Debug)]
pub struct TraitData {
    pub name: Box<str>,
    pub generic_args: Box<[GenericArgType]>,
    // TODO: add bounds / where clauses & associated types
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Scalar {
    bool,
    char,
    uchar,
    Int(IntType),
    UInt(UIntType),
    Float(FloatType),
}

impl Display for Scalar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scalar::bool => write!(f, "bool"),
            Scalar::char => write!(f, "char"),
            Scalar::uchar => write!(f, "uchar"),
            Scalar::Int(int) => write!(f, "{}", int),
            Scalar::UInt(uint) => write!(f, "{}", uint),
            Scalar::Float(float) => write!(f, "{}", float),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum IntType {
    i8,
    i16,
    i32,
    i64,
    isize,
}

impl Display for IntType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntType::i8 => write!(f, "i8"),
            IntType::i16 => write!(f, "i16"),
            IntType::i32 => write!(f, "i32"),
            IntType::i64 => write!(f, "i64"),
            IntType::isize => write!(f, "isize"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum UIntType {
    u8,
    u16,
    u32,
    u64,
    usize,
}

impl Display for UIntType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UIntType::u8 => write!(f, "u8"),
            UIntType::u16 => write!(f, "u16"),
            UIntType::u32 => write!(f, "u32"),
            UIntType::u64 => write!(f, "u64"),
            UIntType::usize => write!(f, "usize"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum FloatType {
    f16,
    f32,
    f64,
}

impl Display for FloatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FloatType::f16 => write!(f, "f16"),
            FloatType::f32 => write!(f, "f32"),
            FloatType::f64 => write!(f, "f64"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RefQual {
    Mut,
    Drop,
}

impl RefQual {
    pub fn repr(qual: Option<Self>) -> &'static str {
        match qual {
            Some(Self::Mut) => "mut ",
            Some(Self::Drop) => "drop ",
            None => "",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PtrQual {
    Mut,
}

impl PtrQual {
    pub fn repr(qual: Option<Self>) -> &'static str {
        match qual {
            Some(Self::Mut) => "mut ",
            None => "",
        }
    }
}

/// TODO: write docs
#[derive(Clone, Copy, Debug)]
pub enum TypeId<I: Interner> {
    Adt(AdtId<I>),
    Scalar(Scalar),
    // NOTE: I'm not entirely sure that this is a good idea, but I want the compiler to understand
    // that `NonZero<T>` can be `{integer}` as well.
    NonZero,
    Slice,
    // Array,
    Ref(Option<RefQual>),
    Ptr(Option<PtrQual>),
    Never,
    // TODO: add function pointers and `dyn Trait`
}

impl<I: Interner> TypeId<I> {
    /// Returns generic argument types if `self` is one of built-in types and `AdtId` otherwise.
    pub fn generic_arg_types<'a>(self) -> Result<&'a [GenericArgType], AdtId<I>> {
        match self {
            TypeId::Adt(id) => Err(id),
            TypeId::NonZero | TypeId::Slice | TypeId::Ref(_) | TypeId::Ptr(_) => {
                Ok(&[GenericArgType::Type])
            }
            TypeId::Scalar(_) | TypeId::Never => Ok(&[]),
        }
    }
}

impl<I: Interner> PartialEq for TypeId<I> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Adt(l0), Self::Adt(r0)) => l0 == r0,
            (Self::Scalar(l0), Self::Scalar(r0)) => l0 == r0,
            (Self::Ref(l0), Self::Ref(r0)) => l0 == r0,
            (Self::Ptr(l0), Self::Ptr(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

/// TODO: write docs
#[derive(Debug)]
pub enum TypeKind<I: Interner> {
    Adt(AdtId<I>, Substitution<I>),
    Scalar(Scalar),
    NonZero(Type<I>),
    Slice(Type<I>),
    Ref(Option<RefQual>, Type<I>),
    Ptr(Option<PtrQual>, Type<I>),
    Never,
}

/// TODO: write docs
#[derive(Debug)]
pub struct TypeData<I: Interner> {
    pub ty_kind: TypeKind<I>,
}

/// TODO: write docs
#[derive(Debug)]
pub enum GenericArgData<I: Interner> {
    Type(Type<I>),
}
