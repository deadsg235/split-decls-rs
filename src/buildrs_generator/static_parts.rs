use proc_macro2::TokenStream;
use quote::quote;
use syn::{Item, ItemUse};
use syn::visit::{self, Visit};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::Context;

pub fn generate_build_rs_macros() -> TokenStream {
    quote! {
        macro_rules! GetToken {
            ($token:tt) => { syn::token::$token::new(proc_macro2::Span::call_site()) };
        }
        macro_rules! mkImplCallVisitor {
            (calls: $init_calls:expr) => {
                struct ImplCallVisitor {
                    calls: std::collections::HashMap<String, std::collections::HashSet<String>>,
                }
                impl ImplCallVisitor {
                    fn new() -> Self {
                        ImplCallVisitor { calls: $init_calls }
                    }
                }
                impl<'ast> syn::visit::Visit<'ast> for ImplCallVisitor {
                    fn visit_macro(&mut self, i: &'ast syn::Macro) {
                        if let Some(path_segment) = i.path.segments.last() {
                            let path_str = path_segment.ident.to_string();
                            if path_str.ends_with("_impl") {
                                if let Some(module_ident) = i.path.segments.first() {
                                    let module_name = module_ident.ident.to_string();
                                    let fn_name = path_str;
                                    self.calls.entry(module_name).or_default().insert(fn_name);
                                }
                            }
                        }
                        syn::visit::visit_macro(self, i);
                    }
                }
            };
        }
    }
}

pub fn generate_build_rs_config_types_for_generated_buildrs() -> TokenStream {
    quote! {
        // Configuration structs (re-defined for standalone build.rs)
        #[derive(Debug, Default, Serialize, Deserialize, Clone)]
        pub struct PatchSpec {
            pub path: PathBuf,
            pub git_reference: Option<String>,
        }

        #[derive(Debug, Default, Serialize, Deserialize, Clone)]
        pub struct StringReplacement {
            pub old: String,
            pub new: String,
        }

        #[derive(Debug, Default, Serialize, Deserialize, Clone)]
        pub struct SplitDeclsConfig {
            pub active_overlay_modules: Vec<String>,
            pub custom_prelude_overlay: Option<String>,
            pub rustc_source_path: Option<PathBuf>,
            pub patches: HashMap<String, Vec<PatchSpec>>,
            pub string_replacements: Option<Vec<StringReplacement>>,
        }

        impl SplitDeclsConfig {
            pub fn load_from_file(path: &Path) -> anyhow::Result<Self> {
                if !path.exists() {
                    return Ok(Self::default());
                }
                let content = std::fs::read_to_string(path)?;
                let config: Self = toml::from_str(&content)?;
                Ok(config)
            }
        }
    }
}

pub fn generate_build_rs_ast_helpers_for_generated_buildrs() -> TokenStream {
    quote! {
        /// Represents a single extracted declaration.
        struct ExtractedDecl {
            name: String,
            kind: String, // e.g., "fn", "struct", "enum"
            content: TokenStream,
        }

        /// Visitor to collect all top-level `use` statements.
        #[derive(Default)]
        struct UseStatementCollector {
            uses: Vec<ItemUse>,
        }

        impl<'ast> Visit<'ast> for UseStatementCollector {
            fn visit_item_use(&mut self, i: &'ast ItemUse) {
                self.uses.push(i.clone());
                visit::visit_item_use(self, i);
            }
        }

        // --- Helper functions for item name/kind (defined locally for the generated build.rs) ---
        fn get_item_name(item: &Item) -> Option<String> {
            match item {
                Item::Fn(item_fn) => Some(item_fn.sig.ident.to_string()),
                Item::Struct(item_struct) => Some(item_struct.ident.to_string()),
                Item::Enum(item_enum) => Some(item_enum.ident.to_string()),
                Item::Const(item_const) => Some(item_const.ident.to_string()),
                Item::Static(item_static) => Some(item_static.ident.to_string()),
                Item::Trait(item_trait) => Some(item_trait.ident.to_string()),
                Item::Type(item_type) => Some(item_type.ident.to_string()),
                Item::Union(item_union) => Some(item_union.ident.to_string()),
                _ => None,
            }
        }

        fn get_item_kind(item: &Item) -> Option<&'static str> {
            match item {
                Item::Fn(_) => Some("fn"),
                Item::Struct(_) => Some("struct"),
                Item::Enum(_) => Some("enum"),
                Item::Const(_) => Some("const"),
                Item::Static(_) => Some("static"),
                Item::Trait(_) => Some("trait"),
                Item::Impl(_) => Some("impl"), // Impl names are complex; this would need refinement
                Item::Type(_) => Some("type"),
                Item::Union(_) => Some("union"),
                _ => None,
            }
        }
    }
}
