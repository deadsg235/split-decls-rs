use anyhow::{Context, Result};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_quote,
    punctuated::Punctuated,
    visit::{self, Visit},
    Ident,
    Item,
    ItemConst,
    ItemEnum,
    ItemFn,
    ItemImpl,
    ItemMacro,
    ItemMod,
    ItemStatic,
    ItemStruct,
    ItemTrait,
    ItemType,
    ItemUnion,
    ItemUse,
    Visibility,
    LitStr,
};
use std::{
    collections::HashMap,
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use toml;
use serde::{Deserialize, Serialize};

// Import the config structs from the main split-decls-rs config module
// Note: We are re-defining these here for the generated build.rs,
// as the generated build.rs is a standalone binary and cannot directly
// import from `crate::config`. This ensures the generated build.rs has
// all necessary type definitions.

/// Defines a single patch file and an optional Git reference for context.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PatchSpec_buildrs {
    /// Path to the patch .rs file, relative to the workspace root.
    pub path: PathBuf,
    /// Optional Git reference (branch, tag, commit hash) associated with this patch.
    /// This indicates the state of the repository for which this patch is relevant.
    pub git_reference: Option<String>,
}

/// Defines a single string replacement operation.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct StringReplacement_buildrs {
    pub old: String,
    pub new: String,
}

/// Configuration for split-decls-rs, defined here for the generated build.rs.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SplitDeclsConfig_buildrs {
    pub active_overlay_modules: Vec<String>,
    pub custom_prelude_overlay: Option<String>,
    pub rustc_source_path: Option<PathBuf>,
    pub patches: HashMap<String, Vec<PatchSpec_buildrs>>,
    pub string_replacements: Option<Vec<StringReplacement_buildrs>>,
}

impl SplitDeclsConfig_buildrs {
    pub fn load_from_file(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
}


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


/// Generates the TokenStream for the target build.rs file.
pub fn generate_build_rs_token_stream(
    old_lib_rs_path: &Path,
    old_build_rs_path: &Path,
    decls_output_dir: &Path,
    crate_name_sanitized: &str,
) -> Result<TokenStream> {
    let old_lib_rs_path_lit = LitStr::new(&old_lib_rs_path.display().to_string(), Span::call_site());
    let old_build_rs_path_lit = LitStr::new(&old_build_rs_path.display().to_string(), Span::call_site());
    let decls_output_dir_lit = LitStr::new(&decls_output_dir.display().to_string(), Span::call_site());
    let crate_name_sanitized_lit = LitStr::new(crate_name_sanitized, Span::call_site());

    let build_rs_token_stream = quote! {
        use std::{
            collections::HashMap,
            collections::HashSet,
            fs,
            path::{Path, PathBuf},
        };

        use anyhow::{Context, Result};
        use proc_macro2::{Span, TokenStream};
        use quote::{quote, ToTokens};
        use syn::{
            parse_quote,
            punctuated::Punctuated,
            visit::{self, Visit},
            Ident,
            Item,
            ItemConst,
            ItemEnum,
            ItemFn,
            ItemImpl,
            ItemMacro,
            ItemMod,
            ItemStatic,
            ItemStruct,
            ItemTrait,
            ItemType,
            ItemUnion,
            ItemUse,
            Visibility,
        };
        use toml;
        use serde::{Deserialize, Serialize};

        #generate_build_rs_config_types()
        #generate_build_rs_macros()
        #generate_build_rs_ast_helpers()

    }; // End of build_rs_token_stream quote! block
    Ok(build_rs_token_stream)
}


fn generate_build_rs_macros() -> TokenStream {
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

