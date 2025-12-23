use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use split_decls_types::SplitDeclsConfig;
use split_decls_rs::process_crate::process_crate;
use std::fs;

fn main() -> Result<()> {
    println!("Starting split-decls-rs tool...");
    
    let args: Vec<String> = std::env::args().collect();
    
    // Parse command line arguments
    let dry_run = args.contains(&"--dry-run".to_string());
    let verbose = args.contains(&"--verbose".to_string());
    
    if dry_run {
        println!("*** Running in DRY-RUN mode. No files will be modified. ***");
    }
    
    // Get target directory from command line args
    let target_path = if args.len() > 1 {
        let mut target_arg = None;
        for (i, arg) in args.iter().enumerate() {
            if !arg.starts_with("--") && i > 0 {
                target_arg = Some(arg.as_str());
                break;
            }
        }
        target_arg.unwrap_or("./")
    } else {
        "./"
    };
    
    let target_path = PathBuf::from(target_path);
    println!("Target path: {}", target_path.display());
    
    // Load configuration
    let global_config_path = PathBuf::from("split-decls-rs.toml");
    let global_config = SplitDeclsConfig::load_from_file(&global_config_path)
        .context("Failed to load global split-decls-rs config")?;
    
    if verbose {
        println!("Global config loaded: {:?}", global_config);
    }
    
    // Check if target is a single crate or directory with multiple crates
    if target_path.join("Cargo.toml").exists() {
        // Single crate
        println!("Processing single crate: {}", target_path.display());
        process_crate(&target_path, &global_config, dry_run)?;
    } else {
        // Directory with multiple crates - find all Cargo.toml files
        println!("Scanning directory for crates: {}", target_path.display());
        scan_and_process_crates(&target_path, &global_config, dry_run)?;
    }
    
    println!("\nSplit-decls-rs tool finished.");
    Ok(())
}

fn scan_and_process_crates(root_path: &Path, global_config: &SplitDeclsConfig, dry_run: bool) -> Result<()> {
    use walkdir::WalkDir;
    
    for entry in WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok()) {
        if entry.file_name() == "Cargo.toml" {
            let cargotoml_path = entry.path();
            let crate_path = cargotoml_path
                .parent()
                .context("Cargo.toml has no parent directory")?;

            // Read Cargo.toml to check if it's a virtual manifest
            let cargo_toml_content = fs::read_to_string(&cargotoml_path)
                .context(format!("Failed to read Cargo.toml from {}", cargotoml_path.display()))?;
            
            #[derive(Debug, serde::Deserialize)]
            struct MinimalCargoToml {
                package: Option<toml::Table>,
            }
            let minimal_cargo_toml: MinimalCargoToml = toml::from_str(&cargo_toml_content)
                .context(format!("Failed to parse Cargo.toml from {}", cargotoml_path.display()))?;

            // Skip virtual manifests (Cargo.toml without a [package] section)
            if minimal_cargo_toml.package.is_none() {
                println!("Skipping virtual manifest: {}", cargotoml_path.display());
                continue;
            }

            let crate_name = crate_path
                .file_name()
                .and_then(|s| s.to_str())
                .context("Could not get crate name")?;

            // Skip self crate
            if crate_name == "split-decls-rs" {
                println!("Skipping self crate: split-decls-rs");
                continue;
            }

            process_crate(crate_path, global_config, dry_run)?;
        }
    }
    Ok(())
}
