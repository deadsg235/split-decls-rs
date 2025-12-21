use anyhow::{Context, Result};
use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use std::fs;
use syn::visit::Visit;
use syn::{self};

use crate::CratePaths;
use split_decls_types::SplitDeclsConfig;

// Import new modules
pub mod use_collector;
pub mod declaration_extractor;
pub mod declaration_writer;
pub mod invocation_generator;

/// Performs the eager splitting of the AST into individual declaration files and generates
/// the `_decl_module_invocation.rs` file.
pub fn split_and_generate_decls(
    syntax_tree: &syn::File,
    paths: &CratePaths,
    config: &SplitDeclsConfig,
    dry_run: bool,
) -> Result<()> {
    // Ensure the output directory exists
    if !dry_run {
        fs::create_dir_all(&paths.decls_output_dir)
            .context(format!("Failed to create directory {}", paths.decls_output_dir.display()))?;
        println!("Created directory: {}", paths.decls_output_dir.display());
    } else {
        println!(
            "Dry-run: Would create directory: {}",
            paths.decls_output_dir.display()
        );
    }

    // Collect use statements
    let mut use_collector_instance = use_collector::UseStatementCollector::default();
    use_collector_instance.visit_file(syntax_tree);
    let common_uses: TokenStream = use_collector_instance.uses.iter().map(|u| u.to_token_stream()).collect();

    let mut collected_module_names: Vec<Ident> = Vec::new();
    let mut item_count = 0; // For generating unique names for impls without explicit paths

    // Extract and split declarations
    for item in &syntax_tree.items {
        if let Some(decl) = declaration_extractor::extract_single_declaration(item, item_count) {
            let module_name_str = format!("{}_decls_{}", paths.crate_name.replace("-", "_"), decl.name);
            let module_name_ident = Ident::new(&module_name_str, Span::call_site());
            collected_module_names.push(module_name_ident.clone());

            declaration_writer::write_declaration_file(
                decl,
                paths,
                config,
                dry_run,
                &common_uses,
                module_name_ident,
            )?;
        }
        item_count += 1;
    }

    // Generate decl_module! invocation
    invocation_generator::generate_decl_module_invocation(collected_module_names, paths, dry_run)?;

    Ok(())
}