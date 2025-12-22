use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket};
use syn::{parse_macro_input, Ident, LitStr, Token, braced, bracketed, parse_quote};
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
    // Output: name = { path = "path = "path/to/crate" }
    let parsed: Punctuated<LitStr, Token![,]> = parse_macro_input!(input with Punctuated::parse_terminated);
    let name_lit = parsed.first().expect("Expected dependency name LitStr").clone();
    let path_lit = parsed.last().expect("Expected dependency path LitStr").clone();
    let name_ident = Ident::new(&name_lit.value(), name_lit.span());
    
    quote! { #name_ident = { path = #path_lit } }.into()
}

// Helper struct for parsing dep_table input
struct DepTableInput {
    name: LitStr,
    _comma_token: Token![,],
    table_content: proc_macro2::TokenStream,
}

impl Parse for DepTableInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: LitStr = input.parse()?;
        let comma_token: Token![,] = input.parse()?;
        let table_content: proc_macro2::TokenStream = input.parse()?; // Reads remaining tokens
        Ok(DepTableInput {
            name,
            _comma_token: comma_token,
            table_content,
        })
    }
}

#[proc_macro]
pub fn dep_table(input: TokenStream) -> TokenStream {
    // Input: "name", version = "1.0", features = ["f1"]  (the rest of the input is the table content)
    // Output: name = { version = "1.0", features = ["f1"] }
    let DepTableInput { name, table_content, .. } = parse_macro_input!(input as DepTableInput);
    let name_ident = Ident::new(&name.value(), name.span());
    
    quote! { #name_ident = { #table_content } }.into()
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
                    Ok(KeyValue::InlineTable(key, content.parse::<proc_macro2::TokenStream>()?))
                } else if lookahead_val.peek(Bracket) {
                    let content;
                    bracketed!(content in input);
                    Ok(KeyValue::List(key, content.parse::<proc_macro2::TokenStream>()?))
                } else {
                    Err(input.error("expected string, braced block, or bracketed list after '='"))
                }
            } else if lookahead.peek(Brace) {
                let content;
                braced!(content in input);
                Ok(KeyValue::Block(key, content.parse::<proc_macro2::TokenStream>()?))
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

    impl Parse for RootItem {
        fn parse(input: ParseStream) -> Result<Self> {
            let id: Ident = input.parse()?;
            let next_lookahead = input.lookahead1();
            if next_lookahead.peek(Brace) {
                // This is a section, e.g., `package { ... }`
                let content;
                braced!(content in input);
                Ok(RootItem::Section(id, content.parse::<proc_macro2::TokenStream>()?))
            } else if next_lookahead.peek(Token![=]) {
                input.parse::<Token![=]>()?;
                let lookahead_val = input.lookahead1();
                if lookahead_val.peek(LitStr) {
                    let val: LitStr = input.parse()?;
                    Ok(RootItem::KeyValue(KeyValue::Simple(id, val)))
                } else if lookahead_val.peek(Brace) {
                    let content;
                    braced!(content in input);
                    Ok(RootItem::KeyValue(KeyValue::InlineTable(id, content.parse::<proc_macro2::TokenStream>()?)))
                } else if lookahead_val.peek(Bracket) {
                    let content;
                    bracketed!(content in input);
                    Ok(RootItem::KeyValue(KeyValue::List(id, content.parse::<proc_macro2::TokenStream>()?)))
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

    // Helper struct for parsing a bracketed list of strings
    pub struct BracketedStringList {
        //_bracket: Bracket, // No need to store the bracket token
        pub list: Punctuated<LitStr, Token![,]>,
    }

    impl Parse for BracketedStringList {
        fn parse(input: ParseStream) -> Result<Self> {
            let content;
            bracketed!(content in input);
            Ok(BracketedStringList {
                //_bracket: bracket,
                list: Punctuated::parse_terminated(&content)?,
            })
        }
    }

    // Parses a list of strings like ["item1", "item2"]
    pub fn parse_string_list(input: proc_macro2::TokenStream) -> Result<Vec<String>> {
        let string_list: BracketedStringList = syn::parse2(input)?;
        Ok(string_list.list.into_iter().map(|s| s.value()).collect())
    }

    // Helper struct for parsing an inline table { key = "value", ... }
    pub struct InlineTable {
        //_brace: Brace, // No need to store the brace token
        pub items: Punctuated<KeyValue, Token![,]>,
    }

    impl Parse for InlineTable {
        fn parse(input: ParseStream) -> Result<Self> {
            let content;
            braced!(content in input);
            Ok(InlineTable {
                //_brace: brace,
                items: Punctuated::parse_terminated(&content)?,
            })
        }
    }

    // Parses an inline table { key = "value", ... }
    pub fn parse_inline_table(input: proc_macro2::TokenStream) -> Result<HashMap<String, String>> {
        let inline_table: InlineTable = syn::parse2(input)?;
        let mut map = HashMap::new();
        for item in inline_table.items {
            if let KeyValue::Simple(key, val) = item {
                map.insert(key.to_string(), val.value());
            } else {
                return Err(syn::Error::new_spanned(item.get_ident_for_err(), "expected simple key-value pairs in inline table"));
            }
        }
        Ok(map)
    }

    // Parses complex dependency table { version = "1.0", features = ["f1"] }
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
                        "workspace" => dep_table.workspace = Some(val.value().parse::<bool>().map_err(|e| syn::Error::new_spanned(&val, format!("invalid boolean for workspace: {}", e)))?),
                        _ => return Err(syn::Error::new_spanned(&key, format!("unsupported key in dependency table: {}", key_str))),
                    }
                },
                KeyValue::List(key, list_tokens) => {
                    let key_str = key.to_string();
                    match key_str.as_str() {
                        "features" => {
                            dep_table.features = Some(parse_string_list(list_tokens)?);
                        },
                        _ => return Err(syn::Error::new_spanned(&key, format!("unsupported list key in dependency table: {}", key_str))),
                    }
                },
                KeyValue::Block(ref key, _) | KeyValue::InlineTable(ref key, _) => {
                    return Err(syn::Error::new_spanned(key, format!("unsupported block/inline table key in dependency table: {}", key.to_string())));
                }
            }
        }
        Ok(dep_table)
    }

    // Parses a HashMap of dependencies, handling both simple string versions and table versions
    // Now it assumes that any helper macros (dep_version, dep_path, dep_table) have already expanded
    pub fn parse_dependencies_map(input: proc_macro2::TokenStream) -> Result<HashMap<String, Dependency>> {
        let toml_section: TomlSection = syn::parse2(input)?; // This expects a braced section of KeyValue items
        let mut deps = HashMap::new();

        for item in toml_section.items {
            match item {
                KeyValue::Simple(key, val) => {
                    deps.insert(key.to_string(), Dependency::Version(val.value()));
                },
                KeyValue::InlineTable(key, table_tokens) => {
                    deps.insert(key.to_string(), Dependency::Table(parse_dependency_table(table_tokens)?));
                },
                _ => return Err(syn::Error::new_spanned(item.get_ident_for_err(), "expected simple version string or inline table for dependency")),
            }
        }
        Ok(deps)
    }

    // Helper to get an Ident for error reporting from a KeyValue
    impl KeyValue {
        fn get_ident_for_err(&self) -> &Ident {
            match self {
                KeyValue::Simple(key, _) => key,
                KeyValue::Block(key, _) => key,
                KeyValue::List(key, _) => key,
                KeyValue::InlineTable(key, _) => key,
            }
        }
    }
}

// Newtype wrapper to implement ToTokens for CargoToml, circumventing orphan rules
struct WrappedCargoToml(CargoToml);

impl ToTokens for WrappedCargoToml {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let serialized = serde_json::to_string(&self.0).expect("Failed to serialize CargoToml to JSON");
        // Emit code that deserializes the JSON string back into a CargoToml instance at runtime
        tokens.extend(quote! {
            {
                let cargo_toml_json = #serialized;
                serde_json::from_str(&cargo_toml_json).expect("Failed to deserialize CargoToml from JSON")
            }
        });
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
                        let section_items: parse_helpers::TomlSection = syn::parse2(block_tokens)
                            .expect(&format!("Failed to parse package section: {}", key));
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
                                    "authors" => package.authors = Some(parse_helpers::parse_string_list(list_tokens).unwrap_or_default()),
                                    "include" => package.include = parse_helpers::parse_string_list(list_tokens).unwrap_or_default(),
                                    "keywords" => package.keywords = parse_helpers::parse_string_list(list_tokens).unwrap_or_default(),
                                    _ => {}
                                }
                            }
                        }
                        cargo_toml.package = Some(package);
                    },
                    "workspace" => {
                        let section_items: parse_helpers::TomlSection = syn::parse2(block_tokens)
                            .expect(&format!("Failed to parse workspace section: {}", key));
                        let mut workspace = Workspace::default();
                        for ws_item in section_items.items {
                            match ws_item {
                                parse_helpers::KeyValue::List(ws_key, list_tokens) => {
                                    let ws_key_str = ws_key.to_string();
                                    match ws_key_str.as_str() {
                                        "members" => workspace.members = parse_helpers::parse_string_list(list_tokens).unwrap_or_default(),
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
                                            workspace.workspace_dependencies = parse_helpers::parse_dependencies_map(ws_block_tokens).unwrap_or_default();
                                        },
                                        "package" => {
                                            // Handle workspace.package block similar to top-level package
                                            let ws_pkg_section_items: parse_helpers::TomlSection = syn::parse2(ws_block_tokens)
                                                .expect(&format!("Failed to parse workspace.package section: {}", ws_key));
                                            let mut ws_package = Package::default();
                                            for ws_pkg_item in ws_pkg_section_items.items {
                                                if let parse_helpers::KeyValue::Simple(ws_pkg_key, ws_pkg_val) = ws_pkg_item {
                                                    let pkg_key_str = ws_pkg_key.to_string();
                                                    match pkg_key_str.as_str() {
                                                        "authors" => ws_package.authors = Some(parse_helpers::parse_string_list(quote!{ [ #ws_pkg_val ] }.into()).unwrap_or_default()), // Assuming single LitStr, wrap in bracketed TokenStream
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
                                                        "authors" => ws_package.authors = Some(parse_helpers::parse_string_list(list_tokens).unwrap_or_default()),
                                                        "include" => ws_package.include = parse_helpers::parse_string_list(list_tokens).unwrap_or_default(),
                                                        "keywords" => ws_package.keywords = parse_helpers::parse_string_list(list_tokens).unwrap_or_default(),
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
                        let section_items: parse_helpers::TomlSection = syn::parse2(block_tokens)
                            .expect(&format!("Failed to parse patch section: {}", key));
                        let mut patch_section = PatchSection::default();
                        for patch_item in section_items.items {
                            if let parse_helpers::KeyValue::Block(patch_key, patch_block_tokens) = patch_item {
                                let patch_key_str = patch_key.to_string();
                                match patch_key_str.as_str() {
                                    "crates-io" => {
                                        patch_section.crates_io = parse_helpers::parse_dependencies_map(patch_block_tokens).unwrap_or_default();
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
                match kv_item {
                    parse_helpers::KeyValue::Block(key, block_tokens) => { // This is for `dependencies { ... }` blocks
                        let key_str = key.to_string();
                        match key_str.as_str() {
                            "dependencies" => {
                                cargo_toml.dependencies = parse_helpers::parse_dependencies_map(block_tokens).unwrap_or_default();
                            },
                            "dev-dependencies" => {
                                cargo_toml.dev_dependencies = parse_helpers::parse_dependencies_map(block_tokens).unwrap_or_default();
                            },
                            "build-dependencies" => {
                                cargo_toml.build_dependencies = parse_helpers::parse_dependencies_map(block_tokens).unwrap_or_default();
                            },
                            _ => {}
                        }
                    },
                    parse_helpers::KeyValue::Simple(ref key, ref val) => {
                        // Handle simple KVs directly at root, if any (unlikely for full sections)
                        // For example, if a helper macro expanded to `foo = "bar"` at root level,
                        // it would be caught here.
                    },
                    parse_helpers::KeyValue::InlineTable(ref key, ref table_tokens) => {
                        // Similar for inline tables expanded from helper macros.
                    },
                    parse_helpers::KeyValue::List(ref key, ref list_tokens) => {
                        // Similar for lists expanded from helper macros.
                    }
                }
            },
        }
    }

    // Now, return the TokenStream that constructs the CargoToml instance
    let output = WrappedCargoToml(cargo_toml);
    output.to_token_stream().into()
}
