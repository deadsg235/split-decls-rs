use anyhow::{Context, Result};
use proc_macro2::Ident;
use quote::quote;
use std::fs;
use syn::punctuated::Punctuated;

use crate::CratePaths;

/// Generates the `_decl_module_invocation.rs` file, which contains the `decl_module!` macro invocation.
pub fn generate_decl_module_invocation(
    collected_module_names: Vec<Ident>,
    paths: &CratePaths,
    dry_run: bool,
) -> Result<()> {
    let decl_module_invocation_args = Punctuated::<Ident, syn::token::Comma>::from_iter(collected_module_names.into_iter());

    let final_decl_module_code = quote! {
        decl_module!(#decl_module_invocation_args);
    };

    let decl_invocation_file_path = paths.decls_output_dir.join("_decl_module_invocation.rs");
    if !dry_run {
        fs::write(&decl_invocation_file_path, final_decl_module_code.to_string())
            .context("Failed to write _decl_module_invocation.rs")?;
        println!(
            "Generated _decl_module_invocation.rs at {}",
            decl_invocation_file_path.display()
        );
    } else {
        println!(
            "Dry-run: Would generate _decl_module_invocation.rs at {}",
            decl_invocation_file_path.display()
        );
    }
    Ok(())
}
