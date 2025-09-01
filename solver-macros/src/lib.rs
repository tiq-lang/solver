use phf::phf_map;
use quote::{ToTokens, TokenStreamExt, quote};
use syn::{
    Expr, Ident, Token, braced, bracketed, parenthesized,
    parse::Parse,
    parse_macro_input,
    punctuated::Punctuated,
    token::{Brace, Bracket, Paren},
};

mod items;
mod keywords;
mod patterns;
mod types;

#[proc_macro]
pub fn impl_patterns(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ImplPatternsInput);
    let interner_tokens = input.interner_expr.to_token_stream();
    let crate_name = if let Some(CrateDetails { crate_name, .. }) = input.crate_details {
        crate_name
    } else {
        Ident::new("crate", proc_macro2::Span::mixed_site())
    };
    let impls = input.impls.iter().map(|elem| {
        let (pat, maybe_trait) = elem.pattern_tokens();
        let primary_ctor = if let Some(tr) = maybe_trait {
            quote! {
                PatternSeq::new_trait_impl(#interner_tokens, &#pat, #tr).unwrap()
            }
        } else {
            quote! {
                PatternSeq::new(#interner_tokens, &#pat).unwrap()
            }
        };
        if elem.has_inference_vars() {
            quote! { #primary_ctor.boxed() }
        } else {
            quote! { ExactPatternSeq::new(#primary_ctor).unwrap().boxed() }
        }
    });
    quote! {
        (
            #(#impls,)*
        )
    }
    .into()
}
