use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use split_decls_types::SplitDeclsConfig;
use walkdir::WalkDir; // Added for finding Cargo.toml files
use rayon::prelude::*; // Added for parallel processing

use crate::eager_splitter; // Added eager_splitter and CratePaths
use crate::paths::{CratePaths, setup_crate_paths};

pub fn process_ecosystem(
    verbose: bool,
    dry_run: bool,
    base_path: &Path,
    recursive: bool,
    global_config: &SplitDeclsConfig,
) -> Result<()> {
    if verbose {
        println!("Scanning ecosystem in: {}", base_path.display());
    }

    let mut cargo_toml_paths: Vec<PathBuf> = Vec::new();

    let walker = if recursive {
        WalkDir::new(base_path)
    } else {
        WalkDir::new(base_path).max_depth(1) // Only check the base_path itself
    };

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.file_name().map_or(false, |f| f == "Cargo.toml") {
            // Basic heuristic to avoid nested Cargo.toml files that are not crate roots
            // This could be improved with more robust logic if needed.
            if let Some(parent) = path.parent() {
                if !parent.ends_with("target") && !parent.ends_with("example_project") && !parent.ends_with("my_test_project") {
                    cargo_toml_paths.push(path.to_path_buf());
                }
            }
        }
    }

    if verbose {
        println!("Found {} Cargo.toml files.", cargo_toml_paths.len());
    }

    // Parallel processing of crates
    cargo_toml_paths.par_iter().try_for_each(|cargo_toml_path| {
        let crate_path = cargo_toml_path.parent().unwrap().to_path_buf();
        if verbose {
            println!("Processing crate: {}", crate_path.display());
        }

        // Setup crate paths
        let paths = setup_crate_paths(&crate_path)?;

        // Perform eager splitting
        eager_splitter::eager_split_crate(&paths, global_config)?;

        Ok(())
    })?;


    Ok(())
}
