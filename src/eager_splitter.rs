use anyhow::{Context, Result};
use proc_macro2::{Ident, Span, TokenStream}; // Added LineColumn
use quote::ToTokens;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf}; // Added Path
use syn::visit::Visit;
use syn::{self, LitStr}; // Added LitStr
use quote::quote; // Added quote

use crate::paths::CratePaths;
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
        
        let file_content = fs::read_to_string(&rust_file)
            .context(format!("Failed to read file: {}", rust_file.display()))?;

        // Sanitize crate name once
        let crate_name_sanitized = paths.crate_name.replace("-", "_").replace(".", "_");

        // Handle parsing errors
        let file_ast = match syn::parse_str::<syn::File>(&file_content) {
            Ok(ast) => ast,
            Err(e) => {
                let file_path_display = rust_file.display();
                eprintln!("Error parsing file {}: {}", file_path_display, e);

                let file_stem = rust_file.file_stem().unwrap().to_string_lossy();
                let error_module_name_str = format!("{}_decls_error_{}", crate_name_sanitized, file_stem.replace("-", "_").replace(".", "_"));
                let error_module_name_ident = Ident::new(&error_module_name_str, Span::call_site());
                collected_module_names.push(error_module_name_ident.clone());

                let error_message = e.to_string();
                let error_line = e.span().start().line;
                let error_column = e.span().start().column;
                
                let file_path_str_lit = LitStr::new(&file_path_display.to_string(), Span::call_site());
                let error_message_lit = LitStr::new(&error_message, Span::call_site());
                let file_content_lit = LitStr::new(&file_content, Span::call_site());
                let crate_name_sanitized_lit = LitStr::new(&crate_name_sanitized, Span::call_site());

                let error_output_tokens = quote! {
                    #[llm_error_message(
                        file = #file_path_str_lit,
                        line = #error_line,
                        column = #error_column,
                        message = #error_message_lit
                    )]
                    #[llm_source_code(
                        #file_content_lit
                    )]
                    #[llm_context(
                        crate_name = #crate_name_sanitized_lit,
                        original_file = #file_path_str_lit
                    )]
                    pub struct #error_module_name_ident; // Dummy item to make it a valid Rust module
                };

                let decl_file_path = paths.decls_output_dir.join(format!("{}.rs", error_module_name_str));
                if !dry_run {
                    fs::write(&decl_file_path, error_output_tokens.to_string())
                        .context(format!("Failed to write error declaration to {}", decl_file_path.display()))?;
                } else {
                    println!("Dry-run: Would write error declaration to {}", decl_file_path.display());
                }
                println!("  (Parsing error captured in {} for later LLM processing)", decl_file_path.display());

                *item_count += 1; // Increment item_count for the error declaration
                return Ok(()); // Continue processing other files
            }
        };
        
        // Get relative path for naming
        let rel_path = rust_file.strip_prefix(&src_dir).unwrap_or(&rust_file);
        let path_str = rel_path.to_string_lossy().replace("/", "_").replace("\\", "_").replace(".rs", "");
        
        for item in &file_ast.items {
            if let Some(decl) = declaration_extractor::extract_single_declaration(item, *item_count) {
                let module_name_str = format!("{}_decls_{}_{}", 
                    crate_name_sanitized, 
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
    
    let mod_content = fs::read_to_string(&mod_file)
        .context(format!("Failed to read module file: {}", mod_file.display()))?;

    let crate_name_sanitized = paths.crate_name.replace("-", "_").replace(".", "_");

    let mod_ast = match syn::parse_str::<syn::File>(&mod_content) {
        Ok(ast) => ast,
        Err(e) => {
            let mod_file_display = mod_file.display();
            eprintln!("Error parsing module file {}: {}", mod_file_display, e);

            let file_stem = mod_file.file_stem().unwrap().to_string_lossy();
            let error_module_name_str = format!("{}_decls_error_{}", crate_name_sanitized, file_stem.replace("-", "_").replace(".", "_"));
            let error_module_name_ident = Ident::new(&error_module_name_str, Span::call_site());
            collected_module_names.push(error_module_name_ident.clone());

            let error_message = e.to_string();
            let error_line = e.span().start().line;
            let error_column = e.span().start().column;
            
            let file_path_str_lit = LitStr::new(&mod_file_display.to_string(), Span::call_site());
            let error_message_lit = LitStr::new(&error_message, Span::call_site());
            let file_content_lit = LitStr::new(&mod_content, Span::call_site());
            let crate_name_sanitized_lit = LitStr::new(&crate_name_sanitized, Span::call_site());

            let error_output_tokens = quote! {
                #[llm_error_message(
                    file = #file_path_str_lit,
                    line = #error_line,
                    column = #error_column,
                    message = #error_message_lit
                )]
                #[llm_source_code(
                    #file_content_lit
                )]
                #[llm_context(
                    crate_name = #crate_name_sanitized_lit,
                    original_file = #file_path_str_lit
                )]
                pub struct #error_module_name_ident;
            };

            let decl_file_path = paths.decls_output_dir.join(format!("{}.rs", error_module_name_str));
            if !dry_run {
                fs::write(&decl_file_path, error_output_tokens.to_string())
                    .context(format!("Failed to write error declaration to {}", decl_file_path.display()))?;
            } else {
                println!("Dry-run: Would write error declaration to {}", decl_file_path.display());
            }
            println!("  (Parsing error captured in {} for later LLM processing)", decl_file_path.display());

            *item_count += 1;
            return Ok(());
        }
    };
    
    for item in &mod_ast.items {
        if let Some(decl) = declaration_extractor::extract_single_declaration(item, *item_count) {
            let full_path = if parent_path.is_empty() {
                mod_name.to_string()
            } else {
                format!("{}_{}", parent_path, mod_name)
            };
            
            let module_name_str = format!("{}_decls_{}_{}", 
                crate_name_sanitized, 
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
    // 1. Parse the original lib.rs
    println!("📖 Parsing lib.rs...");
    let lib_content = fs::read_to_string(&paths.lib_rs_path)
        .context(format!("Failed to read {}", paths.lib_rs_path.display()))?;
    
    println!("🔧 Parsing {} bytes of Rust code...", lib_content.len());
    let syntax_tree: syn::File = match syn::parse_file(&lib_content) {
        Ok(ast) => ast,
        Err(e) => {
            let file_path_display = paths.lib_rs_path.display();
            let error_message = e.to_string();
            let error_line = e.span().start().line;
            let error_column = e.span().start().column;

            return Err(anyhow::anyhow!(
                "Failed to parse lib.rs as Rust code: {}\nFile: {}\nLine: {}, Column: {}\nError: {}",
                file_path_display,
                paths.lib_rs_path.to_string_lossy(),
                error_line,
                error_column,
                error_message
            ));
        }
    };
    
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
    
    let output_lib_path = paths.output_crate_path.join("src").join("lib.rs");
    fs::create_dir_all(output_lib_path.parent().unwrap())
        .context(format!("Failed to create output src directory for lib.rs at {}", output_lib_path.display()))?;

    fs::write(&output_lib_path, new_lib_content)
        .context(format!("Failed to write new lib.rs at {}", output_lib_path.display()))?;
    
    println!("Generated new lib.rs at {}", output_lib_path.display());
    Ok(())
}

/// Generate new build.rs for monitoring changes
fn generate_new_build_rs(paths: &CratePaths) -> Result<()> {
    let build_content = format!(r#"// Generated by split-decls-rs
use anyhow::Result;

fn main() -> Result<()> {{
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=.split-decls-config.toml");
    
    // This build.rs monitors for changes that would require re-running split-decls-rs
    println!("cargo:note=build.rs finished. If split-decls-rs needs to be re-run, changes will be detected.");
    
    Ok(())
}}
"#);
    
    let output_build_path = paths.output_crate_path.join("build.rs");
    fs::create_dir_all(output_build_path.parent().unwrap())
        .context(format!("Failed to create output directory for build.rs at {}", output_build_path.display()))?;

    fs::write(&output_build_path, build_content)
        .context(format!("Failed to write new build.rs at {}", output_build_path.display()))?;
    
    println!("Generated new build.rs at {}", output_build_path.display());
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
    if verbose { // Use config.verbose here, as verbose flag is passed through config
        println!("DEBUG: Collected module names for decl_module!: {:?}", collected_module_names.iter().map(|i| i.to_string()).collect::<Vec<_>>());
    }
    invocation_generator::generate_decl_module_invocation(collected_module_names, paths, dry_run)?;

    println!(); // Add newline after declaration list
    Ok(())
}
