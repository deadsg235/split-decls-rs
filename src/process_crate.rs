use anyhow::{Context, Result};
use std::path::Path;
use std::fs;
use split_decls_types::SplitDeclsConfig;
use crate::paths::setup_crate_paths;
use crate::eager_splitter::eager_split_crate;

/// Process a single crate by applying the split-decls transformation.
pub fn process_crate(crate_path: &Path, global_config: &SplitDeclsConfig, dry_run: bool) -> Result<()> {
    let paths = setup_crate_paths(crate_path)?;
    
    if dry_run {
        println!("DRY RUN: Would process crate at {}", crate_path.display());
        return Ok(());
    }
    
    println!("Processing crate: {}", paths.crate_name);
    
    // Check if lib.rs exists
    if !paths.lib_rs_path.exists() {
        println!("No lib.rs found at {}, skipping", paths.lib_rs_path.display());
        return Ok(());
    }
    
    // Perform eager splitting
    eager_split_crate(&paths, global_config)?;
    
    println!("Successfully processed crate: {}", paths.crate_name);
    Ok(())
}
