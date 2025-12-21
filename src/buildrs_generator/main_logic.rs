use proc_macro2::{Span, TokenStream};
use quote::quote;
//use syn::{LitStr, Item};
use std::path::{Path, PathBuf};
use anyhow::Result;
use std::fs;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize}; // Required for SplitDeclsConfig in the generated build.rs

// These would normally be imported from static_parts.rs but for clarity in this file, we assume them
// For the final build.rs file, these will be embedded directly.
// The actual definitions for these (e.g. SplitDeclsConfig, get_item_name) will be provided by static_parts.rs
// when the token streams are combined.
use syn::LitStr;

pub fn generate_main_logic_token_stream(
    old_lib_rs_path_lit: &LitStr,
    old_build_rs_path_lit: &LitStr,
    decls_output_dir_lit: &LitStr,
    crate_name_sanitized_lit: &LitStr,
) -> TokenStream {
    quote! {
        fn main() -> Result<()> {
            println!("cargo:rerun-if-changed=build.rs");
            println!("cargo:rerun-if-changed{}", #old_lib_rs_path_lit); // oldlib.rs
            println!("cargo:rerun-if-changed{}", #old_build_rs_path_lit); // oldbuild.rs (though unused for now)
            // Watch for changes in the generated config file
            println!("cargo:rerun-if-changed=.split-decls-config.toml");

            // Load configuration for this crate
            let config_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?)
                .join(".split-decls-config.toml");
            let config = SplitDeclsConfig::load_from_file(&config_path)
                .context(format!("Failed to load config from {}", config_path.display()))?;
            println!("Loaded config for this crate: {:?}", config);
            
            fs::create_dir_all(#decls_output_dir_lit).context("Failed to create decls directory")?;

            let mut old_lib_rs_content = fs::read_to_string(#old_lib_rs_path_lit)
                .context("Failed to read oldlib.rs content")?;

            if let Some(replacements) = &config.string_replacements {
                for sr in replacements {
                    old_lib_rs_content = old_lib_rs_content.replace(&sr.old, &sr.new);
                    println!("Applied string replacement: '{}' -> '{}'", sr.old, sr.new);
                }
            }

            let mut syntax_tree = syn::parse_file(&old_lib_rs_content)
                .context("Failed to parse oldlib.rs content")?;

            // Identify patches for the current crate and apply them
            let current_crate_name_for_patch = #crate_name_sanitized_lit.to_string(); // Crate name from format! arg
            let mut patch_items: Vec<Item> = Vec::new();

            if let Some(patches_for_crate) = config.patches.get(&current_crate_name_for_patch) {
                for patch_spec in patches_for_crate {
                    let patch_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?).join(&patch_spec.path);
                    let patch_content = fs::read_to_string(&patch_path)
                        .context(format!("Failed to read patch file {}", patch_path.display()))?;
                    let patch_ast = syn::parse_file(&patch_content)
                        .context(format!("Failed to parse patch file {} AST", patch_path.display()))?;
                    patch_items.extend(patch_ast.items);
                    println!("Loaded patch: {}", patch_path.display());
                    // TODO: Handle git_reference for conditional patch application if needed
                }
            }

            // Apply patches to the syntax_tree using a SynMutVisitor
            // This is a placeholder for more robust AST-based patching.
            struct PatchVisitor {
                patch_items: HashMap<String, Item>,
            }

            impl syn::visit_mut::VisitMut for PatchVisitor {
                fn visit_item_mut(&mut self, i: &mut Item) {
                    if let Some(s_name) = get_item_name(i) {
                        if let Some(p_item) = self.patch_items.get(&s_name) {
                            if get_item_kind(i) == get_item_kind(p_item) {
                                *i = p_item.clone();
                                println!("Applied patch: Replaced item {:?}", s_name);
                            }
                        }
                    }
                    syn::visit_mut::visit_item_mut(self, i);
                }
            }

            let mut patch_map: HashMap<String, Item> = HashMap::new();
            for p_item in patch_items {
                if let Some(name) = get_item_name(&p_item) {
                    patch_map.insert(name, p_item);
                }
            }

            let mut visitor = PatchVisitor { patch_items: patch_map };
            syn::visit_mut::visit_file_mut(&mut visitor, &mut syntax_tree);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use syn::LitStr;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_generate_main_logic_token_stream_basic() {
        let old_lib_rs_path_lit = LitStr::new("/tmp/test_crate/src/oldlib.rs", Span::call_site());
        let old_build_rs_path_lit = LitStr::new("/tmp/test_crate/oldbuild.rs", Span::call_site());
        let decls_output_dir_lit = LitStr::new("/tmp/test_crate/src/decls", Span::call_site());
        let crate_name_sanitized_lit = LitStr::new("test_crate_name", Span::call_site());

        let token_stream = generate_main_logic_token_stream(
            &old_lib_rs_path_lit,
            &old_build_rs_path_lit,
            &decls_output_dir_lit,
            &crate_name_sanitized_lit,
        );

        let code = token_stream.to_string();
        println!("Generated code for main_logic.rs test:\n{}", code);
    }
}
