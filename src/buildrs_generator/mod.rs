use anyhow::Result;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::LitStr;
use std::path::Path;

mod static_parts;
mod main_logic;
pub mod build_script_composer;

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

    let macros_ts = static_parts::generate_build_rs_macros();

    let main_logic_ts = main_logic::generate_main_logic_token_stream(
        &old_lib_rs_path_lit,
        &old_build_rs_path_lit,
        &decls_output_dir_lit,
        &crate_name_sanitized_lit,
    );

    let build_rs_token_stream = quote! {
        use anyhow::Context;
        use anyhow::Result;
        use proc_macro2::{Span, TokenStream};
        use quote::quote;
        use syn::LitStr;
        use std::fs;
        use std::path::{Path, PathBuf};
        use std::collections::HashMap;
        use syn::{self, Item};
        use syn::visit::{self, Visit};
        use syn::visit_mut::{self, VisitMut};
        use split_decls_types::{SplitDeclsConfig, PatchSpec, StringReplacement};
        use toml; // Keep toml for SplitDeclsConfig::load_from_file

        #macros_ts

        #main_logic_ts
    }; // End of build_rs_token_stream quote! block
    Ok(build_rs_token_stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use quote::ToTokens;

    #[test]
    fn test_generate_build_rs_token_stream_basic() {
        let old_lib_rs_path = PathBuf::from("/tmp/test_crate/src/oldlib.rs");
        let old_build_rs_path = PathBuf::from("/tmp/test_crate/oldbuild.rs");
        let decls_output_dir = PathBuf::from("/tmp/test_crate/src/decls");
        let crate_name_sanitized = "test_crate_name";

        let result = generate_build_rs_token_stream(
            &old_lib_rs_path,
            &old_build_rs_path,
            &decls_output_dir,
            crate_name_sanitized,
        );

        assert!(result.is_ok());
        let token_stream = result.unwrap();
        let code = token_stream.to_string();
        println!("Generated code for mod.rs test:\n{}", code);
    }

    #[test]
    fn test_generate_build_rs_token_stream_empty_lib_rs() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
        let old_lib_rs_path = temp_dir.path().join("oldlib.rs");
        let old_build_rs_path = temp_dir.path().join("oldbuild.rs");
        let decls_output_dir = temp_dir.path().join("decls");
        let crate_name_sanitized = "empty_crate";

        // Create an empty oldlib.rs
        std::fs::write(&old_lib_rs_path, "").expect("Failed to write empty oldlib.rs");

        let result = generate_build_rs_token_stream(
            &old_lib_rs_path,
            &old_build_rs_path,
            &decls_output_dir,
            crate_name_sanitized,
        );

        assert!(result.is_ok());
        let token_stream = result.unwrap();
        let code = token_stream.to_string();
        println!("Generated code for empty_lib_rs test:\n{}", code);
        
        let parsed_file: syn::File = syn::parse2(token_stream.clone()).expect("Failed to parse generated TokenStream");

                // Assert that a main function exists and has the correct return type
                let main_fn_item = parsed_file.items.iter().find_map(|item| {
                    if let syn::Item::Fn(item_fn) = item {
                        if item_fn.sig.ident == "main" {
                            Some(item_fn)
                        } else {
                            None
                        }
                    } else {
                        None
                        }
                });
                assert!(main_fn_item.is_some(), "Generated code should contain a main function.");
                let main_fn = main_fn_item.unwrap();
                assert_eq!(main_fn.sig.output.to_token_stream().to_string(), "-> Result < () >", "Main function should return `Result<()>`");
        // Assert that there are no decl_module! macro calls
        let decl_module_call_exists = parsed_file.items.iter().any(|item| {
            if let syn::Item::Macro(item_macro) = item {
                item_macro.mac.path.segments.last().map(|s| s.ident.to_string()) == Some("decl_module".to_string())
            } else {
                false
            }
        });
        assert!(!decl_module_call_exists, "Generated code should not contain `decl_module!` for an empty lib.rs.");

        // Assert that the generated code contains the expected rerun-if-changed directives
        assert!(code.contains("cargo:rerun-if-changed=build.rs"), "Generated code should contain rerun-if-changed for build.rs");
        assert!(code.contains("cargo:rerun-if-changed=.split-decls-config.toml"), "Generated code should contain rerun-if-changed for config file");
    }
}