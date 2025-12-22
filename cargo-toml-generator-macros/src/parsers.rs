use std::collections::HashMap;

use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket};
use syn::{braced, bracketed, Ident, LitStr, Token};

use cargo_toml_generator_types::{Dependency, DependencyTable};

#[derive(Debug)]
pub enum KeyValue {
    Simple(Ident, LitStr),
    Block(Ident, proc_macro2::TokenStream),
    List(Ident, proc_macro2::TokenStream),
    InlineTable(Ident, proc_macro2::TokenStream),
}

impl Parse for KeyValue {
    fn parse(input: ParseStream) -> Result<Self> {
        let key: Ident = input.parse()?;
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            let lookahead_val = input.lookahead1();
            if lookahead_val.peek(LitStr) {
                let val: LitStr = input.parse()?;
                Ok(KeyValue::Simple(key, val))
            } else if lookahead_val.peek(Brace) {
                let content;
                braced!(content in input);
                Ok(KeyValue::InlineTable(
                    key,
                    content.parse::<proc_macro2::TokenStream>()?,
                ))
            } else if lookahead_val.peek(Bracket) {
                let content;
                bracketed!(content in input);
                Ok(KeyValue::List(
                    key,
                    content.parse::<proc_macro2::TokenStream>()?,
                ))
            } else {
                Err(input.error("expected string, braced block, or bracketed list after '='"))
            }
        } else if lookahead.peek(Brace) {
            let content;
            braced!(content in input);
            Ok(KeyValue::Block(
                key,
                content.parse::<proc_macro2::TokenStream>()?,
            ))
        } else {
            Err(input.error("expected '=' or '{'"))
        }
    }
}

impl KeyValue {
    pub fn get_ident_for_err(&self) -> &Ident {
        match self {
            KeyValue::Simple(key, _) => key,
            KeyValue::Block(key, _) => key,
            KeyValue::List(key, _) => key,
            KeyValue::InlineTable(key, _) => key,
        }
    }
}

#[derive(Debug)]
pub enum RootItem {
    Section(Ident, proc_macro2::TokenStream),
    KeyValue(KeyValue),
}

impl Parse for RootItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let id: Ident = input.parse()?;
        let next_lookahead = input.lookahead1();
        if next_lookahead.peek(Brace) {
            let content;
            braced!(content in input);
            Ok(RootItem::Section(
                id,
                content.parse::<proc_macro2::TokenStream>()?,
            ))
        } else if next_lookahead.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            let lookahead_val = input.lookahead1();
            if lookahead_val.peek(LitStr) {
                let val: LitStr = input.parse()?;
                Ok(RootItem::KeyValue(KeyValue::Simple(id, val)))
            } else if lookahead_val.peek(Brace) {
                let content;
                braced!(content in input);
                Ok(RootItem::KeyValue(KeyValue::InlineTable(
                    id,
                    content.parse::<proc_macro2::TokenStream>()?,
                )))
            } else if lookahead_val.peek(Bracket) {
                let content;
                bracketed!(content in input);
                Ok(RootItem::KeyValue(KeyValue::List(
                    id,
                    content.parse::<proc_macro2::TokenStream>()?,
                )))
            } else {
                Err(input.error("expected string, braced block, or bracketed list after '='"))
            }
        } else {
            Err(input.error("expected '{' for section or '=' for key-value pair"))
        }
    }
}

pub struct RootInput {
    pub items: Punctuated<RootItem, Token![,]>,
}

impl Parse for RootInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(RootInput {
            items: Punctuated::parse_terminated(input)?,
        })
    }
}

pub struct TomlSection {
    pub items: Punctuated<KeyValue, Token![,]>,
}

impl Parse for TomlSection {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(TomlSection {
            items: Punctuated::parse_terminated(input)?,
        })
    }
}

pub struct BracketedStringList {
    pub list: Punctuated<LitStr, Token![,]>,
}

impl Parse for BracketedStringList {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        bracketed!(content in input);
        Ok(BracketedStringList {
            list: Punctuated::parse_terminated(&content)?,
        })
    }
}

pub fn parse_string_list(input: proc_macro2::TokenStream) -> Result<Vec<String>> {
    let string_list: BracketedStringList = syn::parse2(input)?;
    Ok(string_list.list.into_iter().map(|s| s.value()).collect())
}

pub struct InlineTable {
    pub items: Punctuated<KeyValue, Token![,]>,
}

impl Parse for InlineTable {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);
        Ok(InlineTable {
            items: Punctuated::parse_terminated(&content)?,
        })
    }
}

pub fn parse_inline_table(input: proc_macro2::TokenStream) -> Result<HashMap<String, String>> {
    let inline_table: InlineTable = syn::parse2(input)?;
    let mut map = HashMap::new();
    for item in inline_table.items {
        if let KeyValue::Simple(key, val) = item {
            map.insert(key.to_string(), val.value());
        } else {
            return Err(syn::Error::new_spanned(
                item.get_ident_for_err(),
                "expected simple key-value pairs in inline table",
            ));
        }
    }
    Ok(map)
}

pub fn parse_dependency_table(input: proc_macro2::TokenStream) -> Result<DependencyTable> {
    let inline_table: InlineTable = syn::parse2(input)?;
    let mut dep_table = DependencyTable::default();

    for item in inline_table.items {
        match item {
            KeyValue::Simple(key, val) => {
                let key_str = key.to_string();
                match key_str.as_str() {
                    "version" => dep_table.version = Some(val.value()),
                    "path" => dep_table.path = Some(val.value()),
                    "git" => dep_table.git = Some(val.value()),
                    "branch" => dep_table.branch = Some(val.value()),
                    "package" => dep_table.package = Some(val.value()),
                    "registry" => dep_table.registry = Some(val.value()),
                    "workspace" => {
                        dep_table.workspace = Some(val.value().parse::<bool>().map_err(|e| {
                            syn::Error::new_spanned(
                                &val,
                                format!("invalid boolean for workspace: {}", e),
                            )
                        })?)
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            &key,
                            format!("unsupported key in dependency table: {}", key_str),
                        ))
                    }
                }
            }
            KeyValue::List(key, list_tokens) => {
                let key_str = key.to_string();
                match key_str.as_str() {
                    "features" => {
                        dep_table.features = Some(parse_string_list(list_tokens)?);
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            &key,
                            format!("unsupported list key in dependency table: {}", key_str),
                        ))
                    }
                }
            }
            KeyValue::Block(ref key, _) | KeyValue::InlineTable(ref key, _) => {
                return Err(syn::Error::new_spanned(
                    key,
                    format!(
                        "unsupported block/inline table key in dependency table: {}",
                        key.to_string()
                    ),
                ));
            }
        }
    }
    Ok(dep_table)
}

pub fn parse_dependencies_map(
    input: proc_macro2::TokenStream,
) -> Result<HashMap<String, Dependency>> {
    let toml_section: TomlSection = syn::parse2(input)?;
    let mut deps = HashMap::new();

    for item in toml_section.items {
        match item {
            KeyValue::Simple(key, val) => {
                deps.insert(key.to_string(), Dependency::Version(val.value()));
            }
            KeyValue::InlineTable(key, table_tokens) => {
                deps.insert(
                    key.to_string(),
                    Dependency::Table(parse_dependency_table(table_tokens)?),
                );
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    item.get_ident_for_err(),
                    "expected simple version string or inline table for dependency",
                ))
            }
        }
    }
    Ok(deps)
}
