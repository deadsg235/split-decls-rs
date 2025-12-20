use anyhow::Result;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::LitStr;
use std::path::Path;

mod static_parts;
mod main_logic;

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

    let config_types_ts = static_parts::generate_build_rs_config_types_for_generated_buildrs();
    let macros_ts = static_parts::generate_build_rs_macros();
    let ast_helpers_ts = static_parts::generate_build_rs_ast_helpers_for_generated_buildrs();

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
        use std::path::{Path, PathBuf};
        use std::fs;
        use std::collections::HashMap;
        use syn::{self, Item};
        use syn::visit::{self, Visit};
        use serde::{Deserialize, Serialize};

        #config_types_ts
        #macros_ts
        #ast_helpers_ts

        #main_logic_ts
    }; // End of build_rs_token_stream quote! block
    Ok(build_rs_token_stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
}