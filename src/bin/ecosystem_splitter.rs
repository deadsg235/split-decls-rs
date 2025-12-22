use anyhow::Result;
use split_decls_rs::process_crate;
use split_decls_types::SplitDeclsConfig;
use std::path::PathBuf;
use std::fs;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let base_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("../../")
    };
    
    let recursive = args.contains(&"--recursive".to_string());
    
    println!("🚀 Starting ecosystem-wide declaration splitting");
    println!("📂 Base path: {}", base_path.display());
    println!("🔄 Recursive: {}", recursive);
    
    let config = SplitDeclsConfig {
        active_overlay_modules: Some(vec![]),
        custom_prelude_overlay: Some("// Split declarations prelude\nuse proc_macro::TokenStream;\nuse quote::quote;\nuse syn::*;".to_string()),
        rustc_source_path: None,
        patches: Some(std::collections::HashMap::new()),
        string_replacements: None,
        crates_io_patches: Some(std::collections::HashMap::new()),
        github_org: None,
        default_branches_to_patch: vec![],
        repo_fork_mapping: std::collections::HashMap::new(),
        workspace_dependencies: std::collections::HashMap::new(),
    };
    
    let mut processed = 0;
    let mut errors = 0;
    
    if recursive {
        process_recursive(&base_path, &config, &mut processed, &mut errors)?;
    } else {
        if let Err(e) = process_crate(&base_path, &config, false) {
            println!("❌ Error processing {}: {}", base_path.display(), e);
            errors += 1;
        } else {
            processed += 1;
        }
    }
    
    println!("✅ Ecosystem transformation complete!");
    println!("📊 Processed: {} crates", processed);
    println!("❌ Errors: {} crates", errors);
    
    Ok(())
}

fn process_recursive(path: &PathBuf, config: &SplitDeclsConfig, processed: &mut i32, errors: &mut i32) -> Result<()> {
    // Look for Cargo.toml files with src/lib.rs
    if path.join("Cargo.toml").exists() && path.join("src/lib.rs").exists() {
        println!("🔧 Processing: {}", path.display());
        match process_crate(path, config, false) {
            Ok(_) => {
                *processed += 1;
                if *processed % 100 == 0 {
                    println!("📈 Progress: {} crates processed", processed);
                }
            }
            Err(e) => {
                println!("❌ Error in {}: {}", path.display(), e);
                *errors += 1;
            }
        }
    }
    
    // Recurse into subdirectories
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    let name = entry_path.file_name().unwrap().to_string_lossy();
                    // Skip common non-source directories
                    if !name.starts_with('.') && name != "target" && name != "node_modules" {
                        process_recursive(&entry_path, config, processed, errors)?;
                    }
                }
            }
        }
    }
    
    Ok(())
}
