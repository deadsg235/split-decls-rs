use proc_macro::TokenStream;

mod macros;
mod parsers;

#[proc_macro]
pub fn dep_version(input: TokenStream) -> TokenStream {
    macros::dep_version_impl(input)
}

#[proc_macro]
pub fn dep_path(input: TokenStream) -> TokenStream {
    macros::dep_path_impl(input)
}

#[proc_macro]
pub fn dep_table(input: TokenStream) -> TokenStream {
    macros::dep_table_impl(input)
}

#[proc_macro]
pub fn workspace_members_list(input: TokenStream) -> TokenStream {
    macros::workspace_members_list_impl(input)
}

#[proc_macro]
pub fn define_root_cargo_toml(input: TokenStream) -> TokenStream {
    macros::define_root_cargo_toml_impl(input)
}