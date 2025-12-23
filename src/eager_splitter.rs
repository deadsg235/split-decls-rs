use anyhow::{Context, Result};
use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf}; // Added Path
use syn::visit::Visit;
use syn::{self};

use crate::CratePaths;
use split_decls_types::SplitDeclsConfig;

// Import new modules
pub mod use_collector;
pub mod declaration_extractor;
pub mod declaration_writer;
pub mod invocation_generator;

/// Recursively finds all .rs files in src directory that cargo would build
fn find_all_rust_files(src_dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>> {
    let mut rust_files = Vec::new();
    
    if !src_dir.exists() {
        return Ok(rust_files);
    }
    
    for entry in std::fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
            rust_files.push(path);
        } else if path.is_dir() {
            // Recursively search subdirectories
            rust_files.extend(find_all_rust_files(&path)?);
        }
    }
    
    Ok(rust_files)
}

/// Processes all .rs files found in src directory
fn process_all_rust_files(
    paths: &CratePaths,
    config: &SplitDeclsConfig,
    collected_module_names: &mut Vec<Ident>,
    item_count: &mut usize,
    common_uses: &TokenStream,
    dry_run: bool,
) -> Result<()> {
    let src_dir = paths.crate_path.join("src");
    let rust_files = find_all_rust_files(&src_dir)?;
    
    for rust_file in rust_files {
        // Skip lib.rs and main.rs as they're handled separately
        if let Some(file_name) = rust_file.file_name() {
            if file_name == "lib.rs" || file_name == "main.rs" {
                continue;
            }
        }
        
        // Skip files in decls directories (generated output)
        if rust_file.to_string_lossy().contains("/decls/") {
            continue;
        }
        
        println!("Processing file: {}", rust_file.display());
        
        let file_content = fs::read_to_string(&rust_file)?;
        let file_ast: syn::File = syn::parse_str(&file_content)?;
        
        // Get relative path for naming
        let rel_path = rust_file.strip_prefix(&src_dir).unwrap_or(&rust_file);
        let path_str = rel_path.to_string_lossy().replace("/", "_").replace("\\", "_").replace(".rs", "");
        
        for item in &file_ast.items {
            if let Some(decl) = declaration_extractor::extract_single_declaration(item, *item_count) {
                let module_name_str = format!("{}_decls_{}_{}", 
                    paths.crate_name.replace("-", "_").replace(".", "_"), 
                    path_str.replace("-", "_").replace(".", "_"),
                    decl.name
                );
                let module_name_ident = Ident::new(&module_name_str, Span::call_site());
                collected_module_names.push(module_name_ident.clone());
                
                declaration_writer::write_declaration_file(
                    decl.clone(),
                    paths,
                    config,
                    dry_run,
                    common_uses,
                    module_name_ident,
                )?;
                print!("{}, ", decl.name);
                *item_count += 1;
            }
        }
    }
    
    Ok(())
}
fn process_module_recursively(
    paths: &CratePaths,
    config: &SplitDeclsConfig,
    mod_name: &str,
    parent_path: &str,
    collected_module_names: &mut Vec<Ident>,
    item_count: &mut usize,
    common_uses: &TokenStream,
    dry_run: bool,
) -> Result<()> {
    let src_dir = paths.crate_path.join("src");
    let mod_file1 = src_dir.join(format!("{}.rs", mod_name));
    let mod_file2 = src_dir.join(mod_name).join("mod.rs");
    
    let mod_file = if mod_file1.exists() {
        mod_file1
    } else if mod_file2.exists() {
        mod_file2
    } else {
        println!("Module file not found for: {}", mod_name);
        return Ok(());
    };
    
    let mod_content = fs::read_to_string(&mod_file)?;
    let mod_ast: syn::File = syn::parse_str(&mod_content)?;
    
    for item in &mod_ast.items {
        if let Some(decl) = declaration_extractor::extract_single_declaration(item, *item_count) {
            let full_path = if parent_path.is_empty() {
                mod_name.to_string()
            } else {
                format!("{}_{}", parent_path, mod_name)
            };
            
            let module_name_str = format!("{}_decls_{}_{}", 
                paths.crate_name.replace("-", "_").replace(".", "_"), 
                full_path.replace("-", "_").replace(".", "_"),
                decl.name
            );
            let module_name_ident = Ident::new(&module_name_str, Span::call_site());
            collected_module_names.push(module_name_ident.clone());
            
            declaration_writer::write_declaration_file(
                decl.clone(),
                paths,
                config,
                dry_run,
                common_uses,
                module_name_ident,
            )?;
            print!("{}, ", decl.name);
            *item_count += 1;
        }
        
        // Recursively process nested modules
        if let syn::Item::Mod(item_mod) = item {
            let nested_mod_name = item_mod.ident.to_string();
            let new_parent_path = if parent_path.is_empty() {
                mod_name.to_string()
            } else {
                format!("{}_{}", parent_path, mod_name)
            };
            
            process_module_recursively(
                paths,
                config,
                &nested_mod_name,
                &new_parent_path,
                collected_module_names,
                item_count,
                common_uses,
                dry_run,
            )?;
        }
    }
    
    Ok(())
}

use std::collections::HashMap; // Added for HashMap

/// Extracts declarations from a crate's lib.rs and returns them as a map.
pub fn extract_declarations_to_map(paths: &CratePaths) -> Result<HashMap<String, TokenStream>> {
    let lib_content = fs::read_to_string(&paths.lib_rs_path)
        .context(format!("Failed to read {}", paths.lib_rs_path.display()))?;
    
    let syntax_tree: syn::File = syn::parse_file(&lib_content)
        .context("Failed to parse lib.rs as Rust code")?;

    let mut extracted_decls: HashMap<String, TokenStream> = HashMap::new();
    let mut item_count = 0; // for unique names if needed

    for item in &syntax_tree.items {
        if let Some(decl) = declaration_extractor::extract_single_declaration(item, item_count) {
            extracted_decls.insert(decl.name, decl.content);
            item_count += 1;
        }
    }
    Ok(extracted_decls)
}

/// Copies extracted declarations to the specified output directory.
pub fn copy_declarations_to_output(
    crate_name: &str,
    declarations: &HashMap<String, String>, // Declarations as name -> TokenStream string
    output_base_path: &Path,
) -> Result<()> {
    let crate_output_dir = output_base_path.join(crate_name).join("src").join("decls");
    std::fs::create_dir_all(&crate_output_dir)
        .context(format!("Failed to create output directory for declarations: {}", crate_output_dir.display()))?;

    for (decl_name, decl_tokens_str) in declarations {
        let file_path = crate_output_dir.join(format!("{}.rs", decl_name));
        std::fs::write(&file_path, decl_tokens_str)
            .context(format!("Failed to write declaration to {}", file_path.display()))?;
    }
    Ok(())
}

/// Main entry point for eager splitting of a crate
pub fn eager_split_crate(paths: &CratePaths, config: &SplitDeclsConfig) -> Result<()> {
    // 1. Backup original files
    backup_original_files(paths)?;
    
    // 2. Parse the original lib.rs
    println!("📖 Parsing lib.rs...");
    let lib_content = fs::read_to_string(&paths.old_lib_rs_path)
        .context(format!("Failed to read {}", paths.old_lib_rs_path.display()))?;
    
    println!("🔧 Parsing {} bytes of Rust code...", lib_content.len());
    let syntax_tree: syn::File = syn::parse_file(&lib_content)
        .context("Failed to parse lib.rs as Rust code")?;
    
    // 3. Split declarations into individual files (to output directory)
    split_and_generate_decls(&syntax_tree, paths, config, false)?;
    
    // 4. Generate new lib.rs in output directory
    generate_output_lib_rs(paths)?;
    
    // 5. Generate new lib.rs in original location (for compatibility)
    generate_new_lib_rs(paths)?;
    
    // 6. Generate new build.rs
    generate_new_build_rs(paths)?;
    
    println!("Eager splitting completed for crate: {}", paths.crate_name);
    println!("Output generated in: {}", paths.decls_output_dir.parent().unwrap().display());
    Ok(())
}

/// Backup original files before modification
fn backup_original_files(paths: &CratePaths) -> Result<()> {
    // Backup lib.rs to oldlib.rs
    if paths.lib_rs_path.exists() {
        fs::copy(&paths.lib_rs_path, &paths.old_lib_rs_path)
            .context(format!("Failed to backup {} to {}", 
                paths.lib_rs_path.display(), paths.old_lib_rs_path.display()))?;
        println!("Backed up lib.rs to oldlib.rs");
    }
    
    // Backup build.rs to oldbuild.rs if it exists
    if paths.build_rs_path.exists() {
        fs::copy(&paths.build_rs_path, &paths.old_build_rs_path)
            .context(format!("Failed to backup {} to {}", 
                paths.build_rs_path.display(), paths.old_build_rs_path.display()))?;
        println!("Backed up build.rs to oldbuild.rs");
    }
    
    Ok(())
}

/// Generate new lib.rs that re-exports the split declarations in the output directory
fn generate_output_lib_rs(paths: &CratePaths) -> Result<()> {
    let output_lib_path = paths.decls_output_dir.parent().unwrap().join("lib.rs");
    let new_lib_content = format!(r#"// Generated by split-decls-rs
// Re-exports all split declarations

pub mod decls {{
    include!("decls/_decl_module_invocation.rs");
}}
pub use decls::*;

// Re-export prelude macros if available

pub use introspector_decl2_macros::*;
"#);
    
    // Ensure the output src directory exists
    fs::create_dir_all(output_lib_path.parent().unwrap())
        .context("Failed to create output src directory")?;
    
    fs::write(&output_lib_path, new_lib_content)
        .context(format!("Failed to write output lib.rs at {}", output_lib_path.display()))?;
    
    println!("Generated output lib.rs at {}", output_lib_path.display());
    Ok(())
}

/// Generate new lib.rs that re-exports the split declarations
fn generate_new_lib_rs(paths: &CratePaths) -> Result<()> {
    let new_lib_content = format!(r#"// Generated by split-decls-rs
// Re-exports all split declarations

pub mod decls {{
    include!("decls/_decl_module_invocation.rs");
}}
pub use decls::*;

// Re-export prelude macros if available

pub use introspector_decl2_macros::*;
"#);
    
    fs::write(&paths.lib_rs_path, new_lib_content)
        .context(format!("Failed to write new lib.rs at {}", paths.lib_rs_path.display()))?;
    
    println!("Generated new lib.rs");
    Ok(())
}

/// Generate new build.rs for monitoring changes
fn generate_new_build_rs(paths: &CratePaths) -> Result<()> {
    let build_content = format!(r#"// Generated by split-decls-rs
use anyhow::Result;

fn main() -> Result<()> {{
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/oldlib.rs");
    println!("cargo:rerun-if-changed=.split-decls-config.toml");
    
    // This build.rs monitors for changes that would require re-running split-decls-rs
    println!("cargo:note=build.rs finished. If split-decls-rs needs to be re-run, changes will be detected.");
    
    Ok(())
}}
"#);
    
    fs::write(&paths.build_rs_path, build_content)
        .context(format!("Failed to write new build.rs at {}", paths.build_rs_path.display()))?;
    
    println!("Generated new build.rs");
    Ok(())
}

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
    println!("🔍 Processing {} items in AST...", syntax_tree.items.len());
    for item in &syntax_tree.items {
        if let Some(decl) = declaration_extractor::extract_single_declaration(item, item_count) {
            let module_name_str = format!("{}_decls_{}", paths.crate_name.replace("-", "_").replace(".", "_"), decl.name);
            let module_name_ident = Ident::new(&module_name_str, Span::call_site());
            collected_module_names.push(module_name_ident.clone());

            declaration_writer::write_declaration_file(
                decl.clone(),
                paths,
                config,
                dry_run,
                &common_uses,
                module_name_ident,
            )?;
            print!("{}, ", decl.name);
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }
        
        // Process modules recursively
        if let syn::Item::Mod(item_mod) = item {
            let mod_name = item_mod.ident.to_string();
            process_module_recursively(
                paths,
                config,
                &mod_name,
                "",
                &mut collected_module_names,
                &mut item_count,
                &common_uses,
                dry_run,
            )?;
        }
        
        item_count += 1;
    }
    
    // Process ALL .rs files in src directory (force include everything cargo would build)
    println!("🔍 Force including all .rs files in src directory...");
    process_all_rust_files(
        paths,
        config,
        &mut collected_module_names,
        &mut item_count,
        &common_uses,
        dry_run,
    )?;

    // Generate decl_module! invocation
    invocation_generator::generate_decl_module_invocation(collected_module_names, paths, dry_run)?;

    println!(); // Add newline after declaration list
    Ok(())
}