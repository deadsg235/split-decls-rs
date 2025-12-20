use anyhow::{Context, Result};
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("Starting split-decls-rs tool...");

    let workspace_root = PathBuf::from("../../"); // Relative to current crate (split-decls-rs)
    let current_crate_name = "split-decls-rs";
    let global_config_path = workspace_root.join("split-decls-rs.toml");
    let global_config = split_decls_rs::config::SplitDeclsConfig::load_from_file(&global_config_path)
        .context("Failed to load global split-decls-rs config")?;
    println!("Global config loaded: {:?}", global_config);

    // Process the main workspace crates first
    println!("\n=== Processing main workspace crates ===");
    split_decls_rs::process_crates_in_path(&workspace_root, current_crate_name, &global_config, false)?;

    // Handle rustc source if specified in config
    if let Some(rustc_source_path_val) = &global_config.rustc_source_path {
        println!("\n=== Processing rustc source at {} ===", rustc_source_path_val.display());

        // For now, assume a fixed repo and reference for rustc
        let rustc_repo_url = "https://github.com/rust-lang/rust";
        // Determine the rustc_reference from global_config.patches if available for rustc
        let mut rustc_reference = "master".to_string(); // Default reference

        if let Some(rustc_patches) = global_config.patches.get("rustc") {
            if let Some(patch_spec) = rustc_patches.iter().find(|ps| ps.git_reference.is_some()) {
                if let Some(ref_str) = &patch_spec.git_reference {
                    rustc_reference = ref_str.clone();
                    println!("Using rustc git reference from config: {}", rustc_reference);
                }
            }
        }
        
        split_decls_rs::handle_git_overlay(rustc_repo_url, rustc_source_path_val, &rustc_reference)?;

        // Now iterate over the crates within the rustc source tree, with is_rustc_source = true
        split_decls_rs::process_crates_in_path(rustc_source_path_val, current_crate_name, &global_config, true)?;
    }

    println!("\nSplit-decls-rs tool finished.");
    Ok(())
}
