use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(UniMacro)]
pub fn unimacro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let expanded = quote! {
        impl #name {
            pub fn new() -> Self {
                Self::default()
            }
        }
    };
    
    TokenStream::from(expanded)
}

pub fn helper_function() -> String {
    "helper".to_string()
}

pub struct HelperStruct {
    pub field: i32,
}

impl HelperStruct {
    pub fn new(field: i32) -> Self {
        Self { field }
    }
}

pub const HELPER_CONST: i32 = 42;
