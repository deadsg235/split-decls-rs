use anyhow::{Context, Result};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::fs;

use crate::buildrs_ast_utils::ExtractedDecl;
use crate::CratePaths;
use split_decls_types::SplitDeclsConfig;

/// Generates the file content for a single declaration and writes it to disk.
pub fn write_declaration_file(
    decl: ExtractedDecl,
    paths: &CratePaths,
    config: &SplitDeclsConfig,
    dry_run: bool,
    common_uses: &TokenStream,
    module_name_ident: Ident,
) -> Result<()> {
    let decl_file_path = paths.decls_output_dir.join(format!("{}.rs", module_name_ident.to_string()));
    
    let custom_prelude = if let Some(ref prelude_str) = config.custom_prelude_overlay {
        prelude_str.parse::<TokenStream>().map_err(|e| anyhow::anyhow!("Failed to parse custom prelude overlay: {}", e))?
    } else {
        quote!{}
    };

    let attr_token_stream = quote! { #[decl_ #module_name_ident] };
    let decl_token_stream = decl.content;

    let attributed_decl_content = quote! {
        #attr_token_stream
        #decl_token_stream
    };

    let file_content = quote! {
        #custom_prelude
        #common_uses
        prelude! {}
        #attributed_decl_content
    };
    
    if !dry_run {
        fs::write(&decl_file_path, file_content.to_string())
            .context(format!("Failed to write to {}", decl_file_path.display()))?;
        println!("Split '{} {}' to {}", decl.kind, decl.name, decl_file_path.display());
    } else {
        println!(
            "Dry-run: Would split '{} {}' to {}",
            decl.kind,
            decl.name,
            decl_file_path.display()
        );
    }
    Ok(())
}
