use proc_macro::TokenStream;
use quote::quote;

#[proc_macro]
pub fn decl_module(_input: TokenStream) -> TokenStream {
    // Empty implementation - just expands to nothing
    quote!().into()
}

#[proc_macro]
pub fn prelude(_input: TokenStream) -> TokenStream {
    // Prelude macro - can insert common imports/code
    quote!().into()
}
