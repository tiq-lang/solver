use syn::{Ident, Token, braced, parse::Parse, token::Brace};

use crate::{
    patterns::ToPatternTokens as _,
    types::{GenericArgs, Path, Type},
};

enum ImplBody {
    Marker(Token![;]),
    Common(Brace),
}

pub struct InherentImpl {
    impl_token: Token![impl],
    implementor: Type,
    body: ImplBody,
}

impl InherentImpl {
    pub fn implementor_ty(&self) -> &Type {
        &self.implementor
    }

    pub fn has_inference_vars(&self) -> bool {
        self.implementor.has_inference_vars()
    }
}

pub struct TraitImpl {
    impl_token: Token![impl],
    implementor: Type,
    as_token: Token![as],
    r#trait: Path,
    body: ImplBody,
}

impl TraitImpl {
    pub fn implementor_ty(&self) -> &Type {
        &self.implementor
    }

    pub fn trait_name(&self) -> &Ident {
        &self.r#trait.ident()
    }

    pub fn trait_args(&self) -> Option<&GenericArgs> {
        self.r#trait.args()
    }

    pub fn has_inference_vars(&self) -> bool {
        self.implementor.has_inference_vars() || self.r#trait.has_inference_vars()
    }
}

pub enum Impl {
    Inherent(InherentImpl),
    Trait(TraitImpl),
}

impl Impl {
    pub fn has_inference_vars(&self) -> bool {
        match self {
            Impl::Inherent(inherent) => inherent.has_inference_vars(),
            Impl::Trait(tr) => tr.has_inference_vars(),
        }
    }
}

impl Parse for ImplBody {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![;]) {
            Ok(Self::Marker(input.parse()?))
        } else if lookahead.peek(Brace) {
            let _content;
            let braces = braced!(_content in input);
            Ok(Self::Common(braces))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for Impl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let impl_token = input.parse()?;
        let implementor = input.parse()?;
        if input.peek(Token![as]) {
            Ok(Self::Trait(TraitImpl {
                impl_token,
                implementor,
                as_token: input.parse()?,
                r#trait: input.parse()?,
                body: input.parse()?,
            }))
        } else {
            Ok(Self::Inherent(InherentImpl {
                impl_token,
                implementor,
                body: input.parse()?,
            }))
        }
    }
}
