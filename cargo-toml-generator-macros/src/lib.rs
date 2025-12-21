use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Ident, LitBool, LitStr, Token, braced, bracketed, parse, parse_quote, parenthesized, Result};
use syn::punctuated::Punctuated;
use syn::token::{Paren, Bracket, Brace};
use std::collections::HashMap;

use cargo_toml_generator_types::{CargoToml, Package, Workspace, Dependency, DependencyTable, PatchSection};

// Helper procedural macros for defining dependencies
// These macros expand to a TokenStream that can then be parsed by define_root_cargo_toml!
// This provides the "another layer" of abstraction.

#[proc_macro]
pub fn dep_version(input: TokenStream) -> TokenStream {
    // Input: "name", "version"
    // Output: name = "version"
    let parsed: Punctuated<LitStr, Token![,]> = parse_macro_input!(input with Punctuated::parse_terminated);
    let name_lit = parsed.first().expect("Expected dependency name LitStr").clone();
    let version_lit = parsed.last().expect("Expected dependency version LitStr").clone();
    let name_ident = Ident::new(&name_lit.value(), name_lit.span());
    
    quote! { #name_ident = #version_lit }.into()
}

#[proc_macro]
pub fn dep_path(input: TokenStream) -> TokenStream {
    // Input: "name", "path/to/crate"
    // Output: name = { path = "path/to/crate" }
    let parsed: Punctuated<LitStr, Token![,]> = parse_macro_input!(input with Punctuated::parse_terminated);
    let name_lit = parsed.first().expect("Expected dependency name LitStr").clone();
    let path_lit = parsed.last().expect("Expected dependency path LitStr").clone();
    let name_ident = Ident::new(&name_lit.value(), name_lit.span());
    
    quote! { #name_ident = { path = #path_lit } }.into()
}

#[proc_macro]
pub fn dep_table(input: TokenStream) -> TokenStream {
    // Input: "name", version = "1.0", features = ["f1"]  (the rest of the input is the table content)
    // Output: name = { version = "1.0", features = ["f1"] }
    let content = parse::ParseBuffer::new(input);
    let name_lit: LitStr = content.parse().expect("Expected dependency name LitStr");
    content.parse::<Token![,]>().expect("Expected comma");
    let table_tokens: proc_macro2::TokenStream = content.parse().expect("Expected dependency table tokens");
    let name_ident = Ident::new(&name_lit.value(), name_lit.span());
    
    quote! { #name_ident = { #table_tokens } }.into()
}

#[proc_macro]
pub fn workspace_members_list(input: TokenStream) -> TokenStream {
    // Input: "member1", "member2", ...
    // Output: ["member1", "member2", ...] (as a TokenStream for parsing)
    let parsed: Punctuated<LitStr, Token![,]> = parse_macro_input!(input with Punctuated::parse_terminated);
    let expanded = quote! {
        [ #parsed ]
    };
    expanded.into()
}


// Custom parsing logic for the macro input structure
mod parse_helpers {
    use super::*;

    #[derive(Debug)]
    pub enum KeyValue {
        Simple(Ident, LitStr), // key = "value"
        Block(Ident, proc_macro2::TokenStream), // key { ... }
        List(Ident, proc_macro2::TokenStream), // key = [ "value", ... ]
        InlineTable(Ident, proc_macro2::TokenStream), // key = { version = "1.0" }
    }

    impl parse::Parse for KeyValue {
        fn parse(input: parse::ParseStream) -> Result<Self> {
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
                    Ok(KeyValue::InlineTable(key, content.into()))
                } else if lookahead_val.peek(Bracket) {
                    let content;
                    bracketed!(content in input);
                    Ok(KeyValue::List(key, content.into()))
                } else {
                    Err(input.error("expected string, braced block, or bracketed list after '='"))
                }
            } else if lookahead.peek(Brace) {
                let content;
                braced!(content in input);
                Ok(KeyValue::Block(key, content.into()))
            }
            else {
                Err(input.error("expected '=' or '{'"))
            }
        }
    }

    // New: Enum for items at the root level of define_root_cargo_toml!, allowing sections or macro calls
    #[derive(Debug)]
    pub enum RootItem {
        Section(Ident, proc_macro2::TokenStream), // e.g., package { ... }, workspace { ... }
        // We expect helper macros to expand to KeyValue, so we can directly parse it.
        KeyValue(KeyValue), 
    }

    impl parse::Parse for RootItem {
        fn parse(input: parse::ParseStream) -> Result<Self> {
            let id: Ident = input.parse()?;
            let next_lookahead = input.lookahead1();
            if next_lookahead.peek(Brace) {
                // This is a section, e.g., `package { ... }`
                let content;
                braced!(content in input);
                Ok(RootItem::Section(id, content.into()))
            } else if next_lookahead.peek(Token![=]) {
                input.parse::<Token![=]>()?;
                let lookahead_val = input.lookahead1();
                if lookahead_val.peek(LitStr) {
                    let val: LitStr = input.parse()?;
                    Ok(RootItem::KeyValue(KeyValue::Simple(id, val)))
                } else if lookahead_val.peek(Brace) {
                    let content;
                    braced!(content in input);
                    Ok(RootItem::KeyValue(KeyValue::InlineTable(id, content.into())))
                } else if lookahead_val.peek(Bracket) {
                    let content;
                    bracketed!(content in input);
                    Ok(RootItem::KeyValue(KeyValue::List(id, content.into())))
                } else {
                    Err(input.error("expected string, braced block, or bracketed list after '='"))
                }
            }
            else {
                Err(input.error("expected '{' for section or '=' for key-value pair"))
            }
        }
    }

    pub struct RootInput {
        pub items: Punctuated<RootItem, Token![,]>,
    }

    impl parse::Parse for RootInput {
        fn parse(input: parse::ParseStream) -> Result<Self> {
            Ok(RootInput {
                items: Punctuated::parse_terminated(input)?,
            })
        }
    }


    pub struct TomlSection {
        pub items: Punctuated<KeyValue, Token![,]>,
    }

    impl parse::Parse for TomlSection {
        fn parse(input: parse::ParseStream) -> Result<Self> {
            Ok(TomlSection {
                items: Punctuated::parse_terminated(input)?,
            })
        }
    }

    // Parses a list of strings like ["item1", "item2"]
    pub fn parse_string_list(input: proc_macro2::TokenStream) -> Result<Vec<String>> {
        let content = parse::ParseBuffer::new(input);
        let lookahead = content.lookahead1();
        if lookahead.peek(Bracket) {
             let inner_content;
             bracketed!(inner_content in content);
             let list: Punctuated<LitStr, Token![,]> = Punctuated::parse_terminated(&inner_content)?;
             Ok(list.into_iter().map(|s| s.value()).collect())
        } else {
             Err(content.error("expected a bracketed list of strings"))
        }
    }

    // Parses an inline table { key = "value", ... }
    pub fn parse_inline_table(input: proc_macro2::TokenStream) -> Result<HashMap<String, String>> {
        let content = parse::ParseBuffer::new(input);
        let items: Punctuated<KeyValue, Token![,] > = Punctuated::parse_terminated(&content)?;
        let mut map = HashMap::new();
        for item in items {
            if let KeyValue::Simple(key, val) = item {
                map.insert(key.to_string(), val.value());
            } else {
                return Err(content.error("expected simple key-value pairs in inline table"));
            }
        }
        Ok(map)
    }

    // Parses complex dependency table { version = "1.0", features = ["f1"] }
    pub fn parse_dependency_table(input: proc_macro2::TokenStream) -> Result<DependencyTable> {
        let content = parse::ParseBuffer::new(input);
        let items: Punctuated<KeyValue, Token![,] > = Punctuated::parse_terminated(&content)?; // Fix Punuated to Punctuated
        let mut dep_table = DependencyTable::default();

        for item in items {
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
                        "workspace" => dep_table.workspace = Some(val.value().parse::<bool>().map_err(|e| content.error(format!("invalid boolean for workspace: {}", e)))?),
                        _ => return Err(content.error(format!("unsupported key in dependency table: {}", key_str))),
                    }
                },
                KeyValue::List(key, list_tokens) => {
                    let key_str = key.to_string();
                    match key_str.as_str() {
                        "features" => {
                            dep_table.features = Some(parse_string_list(list_tokens)?);
                        },
                        _ => return Err(content.error(format!("unsupported list key in dependency table: {}", key_str))),
                    }
                },
                KeyValue::Block(key, _) | KeyValue::InlineTable(key, _) => {
                    return Err(content.error(format!("unsupported block/inline table key in dependency table: {}", key.to_string())));
                }
            }
        }
        Ok(dep_table)
    }

    // Parses a HashMap of dependencies, handling both simple string versions and table versions
    // Now it assumes that any helper macros (dep_version, dep_path, dep_table) have already expanded
    pub fn parse_dependencies_map(input: proc_macro2::TokenStream) -> Result<HashMap<String, Dependency>> {
        let content = parse::ParseBuffer::new(input);
        let items: Punctuated<KeyValue, Token![,] > = Punctuated::parse_terminated(&content)?;
        let mut deps = HashMap::new();

        for item in items {
            match item {
                KeyValue::Simple(key, val) => {
                    deps.insert(key.to_string(), Dependency::Version(val.value()));
                },
                KeyValue::InlineTable(key, table_tokens) => {
                    deps.insert(key.to_string(), Dependency::Table(parse_dependency_table(table_tokens)?));
                },
                _ => return Err(content.error("expected simple version string or inline table for dependency")),
            }
        }
        Ok(deps)
    }
}


#[proc_macro]
pub fn define_root_cargo_toml(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as parse_helpers::RootInput); // Use new RootInput parser
    let mut cargo_toml = CargoToml::default();

    for item in input.items {
        match item {
            parse_helpers::RootItem::Section(key, block_tokens) => {
                let key_str = key.to_string();
                match key_str.as_str() {
                    "package" => {
                        let section_items = parse_macro_input!(block_tokens as parse_helpers::TomlSection);
                        let mut package = Package::default();
                        for pkg_item in section_items.items {
                            if let parse_helpers::KeyValue::Simple(pkg_key, pkg_val) = pkg_item {
                                let pkg_key_str = pkg_key.to_string();
                                match pkg_key_str.as_str() {
                                    "name" => package.name = Some(pkg_val.value()),
                                    "version" => package.version = Some(pkg_val.value()),
                                    "edition" => package.edition = Some(pkg_val.value()),
                                    "description" => package.description = Some(pkg_val.value()),
                                    "homepage" => package.homepage = Some(pkg_val.value()),
                                    "license" => package.license = Some(pkg_val.value()),
                                    "publish" => package.publish = Some(pkg_val.value().parse::<bool>().unwrap_or(false)),
                                    "repository" => package.repository = Some(pkg_val.value()),
                                    "rust-version" => package.rust_version = Some(pkg_val.value()),
                                    "resolver" => package.resolver = Some(pkg_val.value()),
                                    _ => {}
                                }
                            } else if let parse_helpers::KeyValue::List(pkg_key, list_tokens) = pkg_item {
                                let pkg_key_str = pkg_key.to_string();
                                match pkg_key_str.as_str() {
                                    "authors" => package.authors = Some(parse_helpers::parse_string_list(list_tokens).unwrap()),
                                    "include" => package.include = parse_helpers::parse_string_list(list_tokens).unwrap(),
                                    "keywords" => package.keywords = parse_helpers::parse_string_list(list_tokens).unwrap(),
                                    _ => {}
                                }
                            }
                        }
                        cargo_toml.package = Some(package);
                    },
                    "workspace" => {
                        let section_items = parse_macro_input!(block_tokens as parse_helpers::TomlSection);
                        let mut workspace = Workspace::default();
                        for ws_item in section_items.items {
                            match ws_item {
                                parse_helpers::KeyValue::List(ws_key, list_tokens) => {
                                    let ws_key_str = ws_key.to_string();
                                    match ws_key_str.as_str() {
                                        "members" => workspace.members = parse_helpers::parse_string_list(list_tokens).unwrap(),
                                        _ => {}
                                    }
                                },
                                parse_helpers::KeyValue::Simple(ws_key, ws_val) => {
                                    let ws_key_str = ws_key.to_string();
                                    match ws_key_str.as_str() {
                                        "resolver" => workspace.resolver = Some(ws_val.value()),
                                        _ => {}
                                    }
                                },
                                parse_helpers::KeyValue::Block(ws_key, ws_block_tokens) => {
                                    let ws_key_str = ws_key.to_string();
                                    match ws_key_str.as_str() {
                                        "dependencies" => {
                                            workspace.workspace_dependencies = parse_helpers::parse_dependencies_map(ws_block_tokens).unwrap();
                                        },
                                        "package" => {
                                            // Handle workspace.package block similar to top-level package
                                            let ws_pkg_section_items = parse_macro_input!(ws_block_tokens as parse_helpers::TomlSection);
                                            let mut ws_package = Package::default();
                                            for ws_pkg_item in ws_pkg_section_items.items {
                                                if let parse_helpers::KeyValue::Simple(ws_pkg_key, ws_pkg_val) = ws_pkg_item {
                                                    let pkg_key_str = ws_pkg_key.to_string();
                                                    match pkg_key_str.as_str() {
                                                        // Corrected authors parsing for workspace.package
                                                        "authors" => ws_package.authors = Some(parse_helpers::parse_string_list(quote!{ [ #ws_pkg_val ] }.into()).unwrap()), // Assuming single LitStr, wrap in bracketed TokenStream
                                                        "edition" => ws_package.edition = Some(ws_pkg_val.value()),
                                                        "description" => ws_package.description = Some(ws_pkg_val.value()),
                                                        "homepage" => ws_package.homepage = Some(ws_pkg_val.value()),
                                                        "license" => ws_package.license = Some(ws_pkg_val.value()),
                                                        "publish" => ws_package.publish = Some(ws_pkg_val.value().parse::<bool>().unwrap_or(false)),
                                                        "repository" => ws_package.repository = Some(ws_pkg_val.value()),
                                                        "rust-version" => ws_package.rust_version = Some(ws_pkg_val.value()),
                                                        "version" => ws_package.version = Some(ws_pkg_val.value()),
                                                        _ => {}
                                                    }
                                                } else if let parse_helpers::KeyValue::List(pkg_key, list_tokens) = ws_pkg_item {
                                                    let pkg_key_str = pkg_key.to_string();
                                                    match pkg_key_str.as_str() {
                                                        "authors" => ws_package.authors = Some(parse_helpers::parse_string_list(list_tokens).unwrap()),
                                                        "include" => ws_package.include = parse_helpers::parse_string_list(list_tokens).unwrap(),
                                                        "keywords" => ws_package.keywords = Some(parse_helpers::parse_string_list(list_tokens).unwrap()),
                                                        _ => {}
                                                    }
                                                }
                                            }
                                            workspace.package_config = Some(ws_package);
                                        },
                                        _ => {}
                                    }
                                },
                                _ => {}
                            }
                        }
                        cargo_toml.workspace = Some(workspace);
                    },
                    "patch" => {
                        let section_items = parse_macro_input!(block_tokens as parse_helpers::TomlSection);
                        let mut patch_section = PatchSection::default();
                        for patch_item in section_items.items {
                            if let parse_helpers::KeyValue::Block(patch_key, patch_block_tokens) = patch_item {
                                let patch_key_str = patch_key.to_string();
                                match patch_key_str.as_str() {
                                    "crates-io" => {
                                        patch_section.crates_io = parse_helpers::parse_dependencies_map(patch_block_tokens).unwrap();
                                    },
                                    _ => {}
                                }
                            }
                        }
                        cargo_toml.patch = Some(patch_section);
                    },
                    _ => {}
                }
            },
            parse_helpers::RootItem::KeyValue(kv_item) => {
                // This means the root is now also accepting key-value pairs, which are mostly dependencies
                // This will primarily be used for top-level dependencies, dev-dependencies etc.
                if let parse_helpers::KeyValue::Block(key, block_tokens) = kv_item { // RootItem::KeyValue should be Simple, List, InlineTable (expanded from helper macros) not Block
                    let key_str = key.to_string();
                    match key_str.as_str() {
                        "dependencies" => {
                            cargo_toml.dependencies = parse_helpers::parse_dependencies_map(block_tokens).unwrap();
                        },
                        "dev-dependencies" => {
                            cargo_toml.dev_dependencies = parse_helpers::parse_dependencies_map(block_tokens).unwrap();
                        },
                        "build-dependencies" => {
                            cargo_toml.build_dependencies = parse_helpers::parse_dependencies_map(block_tokens).unwrap();
                        },
                        // "workspace.dependencies" is handled within the workspace block
                        _ => {}
                    }
                } else if let parse_helpers::KeyValue::Simple(key, val) = kv_item {
                    // Handle simple KVs directly at root, if any (unlikely for full sections)
                    // For example, if a helper macro expanded to `foo = "bar"` at root level,
                    // it would be caught here.
                } else if let parse_helpers::KeyValue::InlineTable(key, table_tokens) = kv_item {
                    // Similar for inline tables expanded from helper macros.
                }
            },
        }
    }

    let expanded = quote! {
        #cargo_toml
    };

    expanded.into()
}

// Implement ToTokens for our CargoToml structure to allow `quote!(#cargo_toml)`
impl ToTokens for CargoToml {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let serialized = serde_json::to_string(&self).expect("Failed to serialize CargoToml to JSON");
        tokens.extend(quote! {
            {
                let cargo_toml_json = #serialized;
                serde_json::from_str(&cargo_toml_json).expect("Failed to deserialize CargoToml from JSON")
            }
        });
    }
}
