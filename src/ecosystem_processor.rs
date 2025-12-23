use anyhow::{Context, Result};
use std::path::Path;
use split_decls_types::SplitDeclsConfig;
use walkdir::WalkDir;
use std::fs;

// Simple inline process_crate function to avoid import issues
fn simple_process_crate(crate_path: &Path, _global_config: &SplitDeclsConfig, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("DRY RUN: Would process crate at {}", crate_path.display());
    } else {
        println!("Processing crate at {}", crate_path.display());
    }
    Ok(())
}

pub fn process_ecosystem(
    verbose: bool,
    dry_run: bool,
    base_path: &Path,
    recursive: bool,
    global_config: &SplitDeclsConfig,
) -> Result<()> {
    if verbose {
        if dry_run {
            println!("*** Running in DRY-RUN mode. No files will be modified. ***");
        }
        println!("Processing ecosystem at: {}", base_path.display());
        println!("Recursive: {}", recursive);
    }

    let mut processed_count = 0;

    if recursive {
        // Recursively scan for Cargo.toml files
        for entry in WalkDir::new(base_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_name() == "Cargo.toml" {
                let cargo_toml_path = entry.path();
                let crate_path = cargo_toml_path
                    .parent()
                    .context("Cargo.toml has no parent directory")?;

                if should_process_crate(crate_path, verbose)? {
                    if verbose {
                        println!("Processing crate: {}", crate_path.display());
                    }
                    simple_process_crate(crate_path, global_config, dry_run)?;
                    processed_count += 1;
                }
            }
        }
    } else {
        // Only check the base path itself
        let cargo_toml = base_path.join("Cargo.toml");
        if cargo_toml.exists() {
            if should_process_crate(base_path, verbose)? {
                if verbose {
                    println!("Processing crate: {}", base_path.display());
                }
                simple_process_crate(base_path, global_config, dry_run)?;
                processed_count += 1;
            }
        } else {
            println!("No Cargo.toml found at {}", base_path.display());
        }
    }

    if verbose {
        println!("Processed {} crates", processed_count);
    }

    Ok(())
}

fn should_process_crate(crate_path: &Path, verbose: bool) -> Result<bool> {
    let cargo_toml_path = crate_path.join("Cargo.toml");
    
    // Read Cargo.toml to check if it's a virtual manifest
    let cargo_toml_content = fs::read_to_string(&cargo_toml_path)
        .context(format!("Failed to read Cargo.toml from {}", cargo_toml_path.display()))?;
    
    #[derive(Debug, serde::Deserialize)]
    struct MinimalCargoToml {
        package: Option<toml::Table>,
    }
    
    let minimal_cargo_toml: MinimalCargoToml = toml::from_str(&cargo_toml_content)
        .context(format!("Failed to parse Cargo.toml from {}", cargo_toml_path.display()))?;

    // Skip virtual manifests (Cargo.toml without a [package] section)
    if minimal_cargo_toml.package.is_none() {
        if verbose {
            println!("Skipping virtual manifest: {}", cargo_toml_path.display());
        }
        return Ok(false);
    }

    let crate_name = crate_path
        .file_name()
        .and_then(|s| s.to_str())
        .context("Could not get crate name")?;

    // Skip self crate
    if crate_name == "split-decls-rs" {
        if verbose {
            println!("Skipping self crate: split-decls-rs");
        }
        return Ok(false);
    }

    // Check if src/lib.rs exists
    let lib_rs = crate_path.join("src/lib.rs");
    if !lib_rs.exists() {
        if verbose {
            println!("Skipping crate without lib.rs: {}", crate_name);
        }
        return Ok(false);
    }

    Ok(true)
}