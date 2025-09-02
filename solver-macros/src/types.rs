use crate::keywords;
use syn::{
    Ident, Token, bracketed, parenthesized,
    parse::Parse,
    punctuated::Punctuated,
    token::{Bracket, Paren},
};

pub enum Type {
    Grouped(Grouped),
    Never(Never),
    Placeholder(Placeholder),
    Inferred(Inferred),
    Slice(Slice),
    Ref(Ref),
    RefMut(RefMut),
    RefDrop(RefDrop),
    Ptr(Ptr),
    PtrMut(PtrMut),
    Path(Path),
}

pub struct Grouped {
    braces: Paren,
    inner: Box<Type>,
}

impl Grouped {
    pub fn inner_ty(&self) -> &Box<Type> {
        &self.inner
    }
}

pub struct Never(Token![!]);

pub struct Placeholder(Token![_]);

pub struct Inferred(Token![?]);

pub struct Slice {
    brackets: Bracket,
    inner: Box<Type>,
}

impl Slice {
    pub fn element_ty(&self) -> &Box<Type> {
        &self.inner
    }
}

pub struct Ref {
    ref_token: Token![&],
    pointee: Box<Type>,
}

pub struct RefMut {
    ref_token: Token![&],
    mut_token: Token![mut],
    pointee: Box<Type>,
}

pub struct RefDrop {
    ref_token: Token![&],
    drop_token: keywords::drop,
    pointee: Box<Type>,
}

pub struct Ptr {
    ptr_token: Token![*],
    pointee: Box<Type>,
}

pub struct PtrMut {
    ptr_token: Token![*],
    mut_token: keywords::drop,
    pointee: Box<Type>,
}

macro_rules! project_pointee_impl {
    ($($ty:ident),+) => {
        $(
            impl $ty {
                pub fn pointee_ty(&self) -> &Box<Type> {
                    &self.pointee
                }
            }
        )*
    };
}

project_pointee_impl!(Ref, RefMut, RefDrop, Ptr, PtrMut);

pub struct GenericArgs {
    lt_token: Token![<],
    args: Punctuated<Type, Token![,]>,
    gt_token: Token![>],
}

impl GenericArgs {
    pub fn iter(&self) -> impl Iterator<Item = &Type> {
        self.args.iter()
    }
}

pub struct Path {
    ident: Ident,
    generic_args: Option<GenericArgs>,
}

impl Path {
    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn args(&self) -> Option<&GenericArgs> {
        self.generic_args.as_ref()
    }
}

mod parsing_impls {
    use super::*;

    impl Parse for Type {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let lookahead = input.lookahead1();
            if lookahead.peek(Paren) {
                Ok(Self::Grouped(input.parse()?))
            } else if lookahead.peek(Token![!]) {
                Ok(Self::Never(input.parse()?))
            } else if lookahead.peek(Token![_]) {
                Ok(Self::Placeholder(input.parse()?))
            } else if lookahead.peek(Token![?]) {
                Ok(Self::Inferred(input.parse()?))
            } else if lookahead.peek(Bracket) {
                Ok(Self::Slice(input.parse()?))
            } else if lookahead.peek(Token![&]) {
                let ref_token = input.parse()?;
                if input.peek(Token![mut]) {
                    Ok(Self::RefMut(RefMut {
                        ref_token,
                        mut_token: input.parse()?,
                        pointee: input.parse()?,
                    }))
                } else if input.peek(keywords::drop) {
                    Ok(Self::RefDrop(RefDrop {
                        ref_token,
                        drop_token: input.parse()?,
                        pointee: input.parse()?,
                    }))
                } else {
                    Ok(Self::Ref(Ref {
                        ref_token,
                        pointee: input.parse()?,
                    }))
                }
            } else if lookahead.peek(Token![*]) {
                let ptr_token = input.parse()?;
                if input.peek(Token![mut]) {
                    Ok(Self::PtrMut(PtrMut {
                        ptr_token,
                        mut_token: input.parse()?,
                        pointee: input.parse()?,
                    }))
                } else {
                    Ok(Self::Ptr(Ptr {
                        ptr_token,
                        pointee: input.parse()?,
                    }))
                }
            } else if lookahead.peek(Ident) {
                Ok(Self::Path(input.parse()?))
            } else {
                Err(lookahead.error())
            }
        }
    }

    impl Parse for Grouped {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let content;
            Ok(Self {
                braces: parenthesized!(content in input),
                inner: content.parse()?,
            })
        }
    }

    impl Parse for Never {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            input.parse::<Token![!]>().map(|token| Self(token))
        }
    }

    impl Parse for Placeholder {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            input.parse::<Token![_]>().map(|token| Self(token))
        }
    }

    impl Parse for Inferred {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            Ok(Self(input.parse()?))
        }
    }

    impl Parse for Slice {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let content;
            Ok(Self {
                brackets: bracketed!(content in input),
                inner: content.parse()?,
            })
        }
    }

    impl Parse for GenericArgs {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            Ok(Self {
                lt_token: input.parse()?,
                args: {
                    let mut args = Punctuated::new();
                    loop {
                        if input.peek(Token![>]) {
                            break;
                        }
                        let arg: Type = input.parse()?;
                        args.push_value(arg);
                        if input.peek(Token![>]) {
                            break;
                        }
                        let punct: Token![,] = input.parse()?;
                        args.push_punct(punct);
                    }
                    args
                },
                gt_token: input.parse()?,
            })
        }
    }

    impl Parse for Path {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            Ok(Self {
                ident: input.parse()?,
                generic_args: if input.peek(Token![<]) {
                    Some(input.parse()?)
                } else {
                    None
                },
            })
        }
    }
}

mod pattern_impls {
    use super::*;
    use crate::patterns::ToPatternTokens;
    use phf::phf_map;
    use quote::{TokenStreamExt as _, quote};

    impl ToPatternTokens for Type {
        fn to_pattern_tokens(&self, ir_crate: &Ident) -> (usize, proc_macro2::TokenStream) {
            match self {
                Type::Grouped(braced) => braced.to_pattern_tokens(ir_crate),
                Type::Never(never) => never.to_pattern_tokens(ir_crate),
                Type::Placeholder(placeholder) => placeholder.to_pattern_tokens(ir_crate),
                Type::Inferred(inferred) => inferred.to_pattern_tokens(ir_crate),
                Type::Slice(slice) => slice.to_pattern_tokens(ir_crate),
                Type::Ref(reference) => reference.to_pattern_tokens(ir_crate),
                Type::RefMut(ref_mut) => ref_mut.to_pattern_tokens(ir_crate),
                Type::RefDrop(ref_drop) => ref_drop.to_pattern_tokens(ir_crate),
                Type::Ptr(ptr) => ptr.to_pattern_tokens(ir_crate),
                Type::PtrMut(ptr_mut) => ptr_mut.to_pattern_tokens(ir_crate),
                Type::Path(path) => path.to_pattern_tokens(ir_crate),
            }
        }

        fn has_inference_vars(&self) -> bool {
            match self {
                Type::Grouped(braced) => braced.has_inference_vars(),
                Type::Never(never) => never.has_inference_vars(),
                Type::Placeholder(placeholder) => placeholder.has_inference_vars(),
                Type::Inferred(inferred) => inferred.has_inference_vars(),
                Type::Slice(slice) => slice.has_inference_vars(),
                Type::Ref(reference) => reference.has_inference_vars(),
                Type::RefMut(ref_mut) => ref_mut.has_inference_vars(),
                Type::RefDrop(ref_drop) => ref_drop.has_inference_vars(),
                Type::Ptr(ptr) => ptr.has_inference_vars(),
                Type::PtrMut(ptr_mut) => ptr_mut.has_inference_vars(),
                Type::Path(path) => path.has_inference_vars(),
            }
        }
    }

    impl ToPatternTokens for Grouped {
        fn to_pattern_tokens(&self, ir_crate: &Ident) -> (usize, proc_macro2::TokenStream) {
            self.inner.to_pattern_tokens(ir_crate)
        }

        fn has_inference_vars(&self) -> bool {
            self.inner.has_inference_vars()
        }
    }

    impl ToPatternTokens for Never {
        fn to_pattern_tokens(&self, ir_crate: &Ident) -> (usize, proc_macro2::TokenStream) {
            (
                1,
                quote! {
                    #ir_crate::patterns::PatternElement::TypeConstructor {
                        args_length: 0,
                        type_id: #ir_crate::primitives::TypeId::Never,
                    },
                },
            )
        }

        fn has_inference_vars(&self) -> bool {
            false
        }
    }

    impl ToPatternTokens for Placeholder {
        fn to_pattern_tokens(&self, ir_crate: &Ident) -> (usize, proc_macro2::TokenStream) {
            (
                1,
                quote! {
                    #ir_crate::patterns::PatternElement::TypePlaceholder,
                },
            )
        }

        fn has_inference_vars(&self) -> bool {
            false
        }
    }

    impl ToPatternTokens for Inferred {
        fn to_pattern_tokens(&self, ir_crate: &Ident) -> (usize, proc_macro2::TokenStream) {
            (
                1,
                quote! {
                    #ir_crate::patterns::PatternElement::InferredType,
                },
            )
        }

        fn has_inference_vars(&self) -> bool {
            true
        }
    }

    impl ToPatternTokens for Slice {
        fn to_pattern_tokens(&self, ir_crate: &Ident) -> (usize, proc_macro2::TokenStream) {
            let (len, tokens) = self.inner.to_pattern_tokens(ir_crate);
            (
                len + 1,
                quote! {
                    #ir_crate::patterns::PatternElement::TypeConstructor {
                        args_length: #len,
                        type_id: #ir_crate::primitives::TypeId::Slice,
                    },
                    #tokens
                },
            )
        }

        fn has_inference_vars(&self) -> bool {
            self.inner.has_inference_vars()
        }
    }

    impl ToPatternTokens for Ref {
        fn to_pattern_tokens(&self, ir_crate: &Ident) -> (usize, proc_macro2::TokenStream) {
            let (len, tokens) = self.pointee.to_pattern_tokens(ir_crate);
            (
                len + 1,
                quote! {
                    #ir_crate::patterns::PatternElement::TypeConstructor {
                        args_length: #len,
                        type_id: #ir_crate::primitives::TypeId::Ref(None),
                    },
                    #tokens
                },
            )
        }

        fn has_inference_vars(&self) -> bool {
            self.pointee.has_inference_vars()
        }
    }

    impl ToPatternTokens for RefMut {
        fn to_pattern_tokens(&self, ir_crate: &Ident) -> (usize, proc_macro2::TokenStream) {
            let (len, tokens) = self.pointee.to_pattern_tokens(ir_crate);
            (
                len + 1,
                quote! {
                    #ir_crate::patterns::PatternElement::TypeConstructor {
                        args_length: #len,
                        type_id: #ir_crate::primitives::TypeId::Ref(
                            Some(#ir_crate::primitives::RefQual::Mut)
                        ),
                    },
                    #tokens
                },
            )
        }

        fn has_inference_vars(&self) -> bool {
            self.pointee.has_inference_vars()
        }
    }

    impl ToPatternTokens for RefDrop {
        fn to_pattern_tokens(&self, ir_crate: &Ident) -> (usize, proc_macro2::TokenStream) {
            let (len, tokens) = self.pointee.to_pattern_tokens(ir_crate);
            (
                len + 1,
                quote! {
                    #ir_crate::patterns::PatternElement::TypeConstructor {
                        args_length: #len,
                        type_id: #ir_crate::primitives::TypeId::Ref(
                            Some(#ir_crate::primitives::RefQual::Drop)
                        ),
                    },
                    #tokens
                },
            )
        }

        fn has_inference_vars(&self) -> bool {
            self.pointee.has_inference_vars()
        }
    }

    impl ToPatternTokens for Ptr {
        fn to_pattern_tokens(&self, ir_crate: &Ident) -> (usize, proc_macro2::TokenStream) {
            let (len, tokens) = self.pointee.to_pattern_tokens(ir_crate);
            (
                len + 1,
                quote! {
                    #ir_crate::patterns::PatternElement::TypeConstructor {
                        args_length: #len,
                        type_id: #ir_crate::primitives::TypeId::Ptr(None),
                    },
                    #tokens
                },
            )
        }

        fn has_inference_vars(&self) -> bool {
            self.pointee.has_inference_vars()
        }
    }

    impl ToPatternTokens for PtrMut {
        fn to_pattern_tokens(&self, ir_crate: &Ident) -> (usize, proc_macro2::TokenStream) {
            let (len, tokens) = self.pointee.to_pattern_tokens(ir_crate);
            (
                len + 1,
                quote! {
                    #ir_crate::patterns::PatternElement::TypeConstructor {
                        args_length: #len,
                        type_id: #ir_crate::primitives::TypeId::Ptr(
                            Some(#ir_crate::primitives::PtrQual::Mut)
                        ),
                    },
                    #tokens
                },
            )
        }

        fn has_inference_vars(&self) -> bool {
            self.pointee.has_inference_vars()
        }
    }

    #[allow(non_camel_case_types)]
    enum ReservedType {
        NonZero,
        bool,
        char,
        uchar,
        i8,
        i16,
        i32,
        i64,
        isize,
        u8,
        u16,
        u32,
        u64,
        usize,
        f16,
        f32,
        f64,
    }

    impl Path {
        fn reserved_type_pattern_tokens(
            &self,
            ir_crate: &Ident,
        ) -> Option<proc_macro2::TokenStream> {
            const RESERVED_TYPES: phf::Map<&'static str, ReservedType> = phf_map! {
                "NonZero" => ReservedType::NonZero,
                "bool" => ReservedType::bool,
                "char" => ReservedType::char,
                "uchar" => ReservedType::uchar,
                "i8" => ReservedType::i8,
                "i16" => ReservedType::i16,
                "i32" => ReservedType::i32,
                "i64" => ReservedType::i64,
                "isize" => ReservedType::isize,
                "u8" => ReservedType::u8,
                "u16" => ReservedType::u16,
                "u32" => ReservedType::u32,
                "u64" => ReservedType::u64,
                "usize" => ReservedType::usize,
                "f16" => ReservedType::f16,
                "f32" => ReservedType::f32,
                "f64" => ReservedType::f64,
            };

            let ty = self.ident.to_string();
            Some(match RESERVED_TYPES.get(&ty)? {
                ReservedType::NonZero => quote! { #ir_crate::primitives::TypeId::NonZero },
                ReservedType::bool => quote! {
                    #ir_crate::primitives::TypeId::Scalar(#ir_crate::primitives::Scalar::bool)
                },
                ReservedType::char => quote! {
                    #ir_crate::primitives::TypeId::Scalar(#ir_crate::primitives::Scalar::char)
                },
                ReservedType::uchar => quote! {
                    #ir_crate::primitives::TypeId::Scalar(#ir_crate::primitives::Scalar::uchar)
                },
                ReservedType::i8 => quote! {
                    #ir_crate::primitives::TypeId::Scalar(
                        #ir_crate::primitives::Scalar::Int(#ir_crate::primitives::IntType::i8)
                    )
                },
                ReservedType::i16 => quote! {
                    #ir_crate::primitives::TypeId::Scalar(
                        #ir_crate::primitives::Scalar::Int(#ir_crate::primitives::IntType::i16)
                    )
                },
                ReservedType::i32 => quote! {
                    #ir_crate::primitives::TypeId::Scalar(
                        #ir_crate::primitives::Scalar::Int(#ir_crate::primitives::IntType::i32)
                    )
                },
                ReservedType::i64 => quote! {
                    #ir_crate::primitives::TypeId::Scalar(
                        #ir_crate::primitives::Scalar::Int(#ir_crate::primitives::IntType::i64)
                    )
                },
                ReservedType::isize => quote! {
                    #ir_crate::primitives::TypeId::Scalar(
                        #ir_crate::primitives::Scalar::Int(#ir_crate::primitives::IntType::isize)
                    )
                },
                ReservedType::u8 => quote! {
                    #ir_crate::primitives::TypeId::Scalar(
                        #ir_crate::primitives::Scalar::UInt(#ir_crate::primitives::UIntType::u8)
                    )
                },
                ReservedType::u16 => quote! {
                    #ir_crate::primitives::TypeId::Scalar(
                        #ir_crate::primitives::Scalar::UInt(#ir_crate::primitives::UIntType::u16)
                    )
                },
                ReservedType::u32 => quote! {
                    #ir_crate::primitives::TypeId::Scalar(
                        #ir_crate::primitives::Scalar::UInt(#ir_crate::primitives::UIntType::u32)
                    )
                },
                ReservedType::u64 => quote! {
                    #ir_crate::primitives::TypeId::Scalar(
                        #ir_crate::primitives::Scalar::UInt(#ir_crate::primitives::UIntType::u64)
                    )
                },
                ReservedType::usize => quote! {
                    #ir_crate::primitives::TypeId::Scalar(
                        #ir_crate::primitives::Scalar::UInt(#ir_crate::primitives::UIntType::usize)
                    )
                },
                ReservedType::f16 => quote! {
                    #ir_crate::primitives::TypeId::Scalar(
                        #ir_crate::primitives::Scalar::Float(#ir_crate::primitives::FloatType::f16)
                    )
                },
                ReservedType::f32 => quote! {
                    #ir_crate::primitives::TypeId::Scalar(
                        #ir_crate::primitives::Scalar::Float(#ir_crate::primitives::FloatType::f32)
                    )
                },
                ReservedType::f64 => quote! {
                    #ir_crate::primitives::TypeId::Scalar(
                        #ir_crate::primitives::Scalar::Float(#ir_crate::primitives::FloatType::f64)
                    )
                },
            })
        }
    }

    impl ToPatternTokens for Path {
        fn to_pattern_tokens(&self, ir_crate: &Ident) -> (usize, proc_macro2::TokenStream) {
            let (mut len, mut tokens) = (0, proc_macro2::TokenStream::new());
            if let Some(args) = &self.generic_args {
                tokens.append_all(args.iter().map(|ty| {
                    let (arg_len, arg_tokens) = ty.to_pattern_tokens(ir_crate);
                    len += arg_len;
                    arg_tokens
                }));
            }
            if let Some(ty) = self.reserved_type_pattern_tokens(ir_crate) {
                (
                    len + 1,
                    quote! {
                        #ir_crate::patterns::PatternElement::TypeConstructor {
                            args_length: #len,
                            type_id: #ty,
                        },
                        #tokens
                    },
                )
            } else {
                let adt_id = &self.ident;
                (
                    len + 1,
                    quote! {
                        #ir_crate::patterns::PatternElement::TypeConstructor {
                            args_length: #len,
                            type_id: #ir_crate::primitives::TypeId::Adt(#adt_id),
                        },
                        #tokens
                    },
                )
            }
        }

        fn has_inference_vars(&self) -> bool {
            if let Some(args) = &self.generic_args {
                args.args.iter().any(|arg| arg.has_inference_vars())
            } else {
                false
            }
        }
    }
}
