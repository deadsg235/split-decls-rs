use cargo_toml_generator_types::{
    CargoToml, Dependency, Package, PatchSection, Workspace,
};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Ident, LitStr, Token};

use crate::parsers::{
    parse_dependencies_map, parse_string_list, RootInput, TomlSection,
};

pub fn dep_version_impl(input: TokenStream) -> TokenStream {
    let parsed: Punctuated<LitStr, Token![,]> =
        parse_macro_input!(input with Punctuated::parse_terminated);
    let name_lit = parsed.first().expect("Expected dependency name LitStr").clone();
    let version_lit = parsed
        .last()
        .expect("Expected dependency version LitStr")
        .clone();
    let name_ident = Ident::new(&name_lit.value(), name_lit.span());

    quote! { #name_ident = #version_lit }.into()
}

pub fn dep_path_impl(input: TokenStream) -> TokenStream {
    let parsed: Punctuated<LitStr, Token![,]> =
        parse_macro_input!(input with Punctuated::parse_terminated);
    let name_lit = parsed.first().expect("Expected dependency name LitStr").clone();
    let path_lit = parsed
        .last()
        .expect("Expected dependency path LitStr")
        .clone();
    let name_ident = Ident::new(&name_lit.value(), name_lit.span());

    quote! { #name_ident = { path = #path_lit } }.into()
}

struct DepTableInput {
    name: LitStr,
    _comma_token: Token![,],
    table_content: proc_macro2::TokenStream,
}

impl Parse for DepTableInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: LitStr = input.parse()?;
        let comma_token: Token![,] = input.parse()?;
        let table_content: proc_macro2::TokenStream = input.parse()?;
        Ok(DepTableInput {
            name,
            _comma_token: comma_token,
            table_content,
        })
    }
}

pub fn dep_table_impl(input: TokenStream) -> TokenStream {
    let DepTableInput {
        name,
        table_content,
        ..
    } = parse_macro_input!(input as DepTableInput);
    let name_ident = Ident::new(&name.value(), name.span());

    quote! { #name_ident = { #table_content } }.into()
}

pub fn workspace_members_list_impl(input: TokenStream) -> TokenStream {
    let parsed: Punctuated<LitStr, Token![,]> =
        parse_macro_input!(input with Punctuated::parse_terminated);
    let expanded = quote! {
        [ #parsed ]
    };
    expanded.into()
}

struct WrappedCargoToml(CargoToml);

impl ToTokens for WrappedCargoToml {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let serialized =
            serde_json::to_string(&self.0).expect("Failed to serialize CargoToml to JSON");
        tokens.extend(quote! {
            {
                let cargo_toml_json = #serialized;
                serde_json::from_str(&cargo_toml_json).expect("Failed to deserialize CargoToml from JSON")
            }
        });
    }
}

pub fn define_root_cargo_toml_impl(input: TokenStream) -> TokenStream {
    //let input = parse_macro_input!(input as RootInput);
    let mut cargo_toml = CargoToml::default();

    let mut package = Package::default();
    package.name = Some("split-decls-rs".to_string());
    package.version = Some("0.1.0".to_string());
    package.edition = Some("2024".to_string());
    package.authors = Some(vec![]);
    package.description = Some("".to_string());
    package.homepage = Some("".to_string());
    package.include = vec![];
    package.keywords = vec![];
    package.license = Some("AGPL 3.0".to_string());
    package.publish = Some(false);
    package.repository = Some("".to_string());
    package.rust_version = Some("1.85.0".to_string());
    cargo_toml.package = Some(package);

    let output = WrappedCargoToml(cargo_toml);
    output.to_token_stream().into()
}
