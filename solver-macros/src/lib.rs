mod items;
mod keywords;
mod patterns;
mod types;

#[proc_macro]
pub fn impl_patterns(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    patterns::impl_patterns(input)
}
