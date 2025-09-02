use crate::items::{Impl, InherentImpl, TraitImpl};
use quote::quote;
use syn::{Expr, Ident, Token, braced, parse::Parse, parse_macro_input, token::Brace};

pub trait ToPatternTokens {
    fn to_pattern_tokens(&self, ir_crate: &Ident) -> (usize, proc_macro2::TokenStream);

    fn has_inference_vars(&self) -> bool;
}

impl InherentImpl {
    fn to_pattern_tokens(&self, ir_crate: &Ident) -> proc_macro2::TokenStream {
        let implementor = self.implementor_ty().to_pattern_tokens(ir_crate).1;
        quote! {
            [ #implementor ]
        }
    }
}

impl TraitImpl {
    fn to_pattern_tokens(&self, ir_crate: &Ident) -> proc_macro2::TokenStream {
        let implementor = self.implementor_ty().to_pattern_tokens(ir_crate).1;
        if let Some(args) = self.trait_args() {
            let args = args.iter().map(|arg| arg.to_pattern_tokens(ir_crate).1);
            quote! {
                [ #implementor #( #args )* ]
            }
        } else {
            quote! {
                [ #implementor ]
            }
        }
    }
}

struct UseCrate {
    use_token: Token![use],
    crate_token: Token![crate],
    crate_name: Ident,
    comma: Token![,],
}

impl Parse for UseCrate {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            use_token: input.parse()?,
            crate_token: input.parse()?,
            crate_name: input.parse()?,
            comma: input.parse()?,
        })
    }
}

struct ImplPatternsInput {
    use_crate: Option<UseCrate>,
    interner_expr: Expr,
    comma: Token![,],
    braces: Brace,
    impls: Vec<Impl>,
}

impl Parse for ImplPatternsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            use_crate: if input.peek(Token![use]) {
                Some(input.parse()?)
            } else {
                None
            },
            interner_expr: input.parse()?,
            comma: input.parse()?,
            braces: braced!(content in input),
            impls: {
                let mut impls = Vec::new();
                while !content.is_empty() {
                    let r#impl: Impl = content.parse()?;
                    impls.push(r#impl);
                }
                impls
            },
        })
    }
}

pub fn impl_patterns(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ImplPatternsInput {
        use_crate,
        interner_expr: ref interner,
        impls,
        ..
    } = parse_macro_input!(input as ImplPatternsInput);
    let ir_crate = use_crate.map_or_else(
        || Ident::new("crate", proc_macro2::Span::mixed_site()),
        |use_crate| use_crate.crate_name,
    );
    let impls = impls.iter().map(|item| {
        let primary_ctor = match item {
            Impl::Inherent(inherent) => {
                let pat_tokens = inherent.to_pattern_tokens(&ir_crate);
                quote! {
                    #ir_crate::patterns::PatternSeq::new(
                        #interner,
                        &#pat_tokens
                    ).unwrap()
                }
            }
            Impl::Trait(tr) => {
                let pat_tokens = tr.to_pattern_tokens(&ir_crate);
                let trait_name = tr.trait_name();
                quote! {
                    #ir_crate::patterns::PatternSeq::new_trait_impl(
                        #interner,
                        &#pat_tokens,
                        #trait_name
                    ).unwrap()
                }
            }
        };
        if item.has_inference_vars() {
            quote! { #primary_ctor.boxed() }
        } else {
            quote! {
                #ir_crate::patterns::ExactPatternSeq::new(#primary_ctor)
                    .unwrap()
                    .boxed()
            }
        }
    });
    quote! {
        (
            #( #impls, )*
        )
    }
    .into()
}
