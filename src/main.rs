use anyhow::{Context, Result};
use std::path::PathBuf;
use split_decls_rs::patch_config::PatchConfig;
use split_decls_types::SplitDeclsConfig;
use split_decls_rs::generate_wrapped_workspace::generate_wrapped_workspace;
use toml;
use std::collections::HashMap;
use std::fs;
use cargo_toml_generator_types::{CargoToml, Dependency}; // Import the new CargoToml and Dependency structs

fn main() -> Result<()> {
    println!("Starting split-decls-rs tool...");

    // Hardcode values for minimal execution
    let dry_run = false;
    let _verbose = false;
    let wrapped_workspace_output_dir = PathBuf::from("output");
    let patch_config_path_str = "patch.toml";

    if dry_run {
        println!("*** Running in DRY-RUN mode. No files will be modified. ***");
    }

    let workspace_root = PathBuf::from("./"); // Relative to current crate (split-decls-rs) - now project root
    let global_config_path = workspace_root.join("split-decls-rs.toml");
    // Change to read the generated Cargo.toml
    let root_cargo_toml_path = workspace_root.join("output/Cargo.toml");
    let mut global_config = SplitDeclsConfig::load_from_file(&global_config_path)
        .context("Failed to load global split-decls-rs config")?;
    println!("Global config loaded: {:?}", global_config);

    // --- Load generated Cargo.toml and extract workspace dependencies ---
    let root_cargo_toml_content = fs::read_to_string(&root_cargo_toml_path)
        .context(format!("Failed to read generated Cargo.toml from {}", root_cargo_toml_path.display()))?;

    // Use the new CargoToml struct
    let root_cargo_toml: CargoToml = toml::from_str(&root_cargo_toml_content)
        .context(format!("Failed to parse generated Cargo.toml from {}", root_cargo_toml_path.display()))?;

    // Populate workspace dependencies from the generated CargoToml
    if let Some(workspace_section) = root_cargo_toml.workspace {
        global_config.workspace_dependencies.extend(workspace_section.workspace_dependencies);
    }
    // Also extend with top-level dependencies, if any, for compatibility
    global_config.workspace_dependencies.extend(root_cargo_toml.dependencies);
    global_config.workspace_dependencies.extend(root_cargo_toml.dev_dependencies);
    global_config.workspace_dependencies.extend(root_cargo_toml.build_dependencies); // Include build-dependencies as well

    // Add dependencies from [patch] sections to workspace_dependencies
    if let Some(patch_section) = root_cargo_toml.patch {
        global_config.workspace_dependencies.extend(patch_section.crates_io);
    }
    // --- END new logic ---

    let patch_config_path = PathBuf::from(patch_config_path_str);
    let patch_config = PatchConfig::load_from_file(&patch_config_path)
        .context(format!("Failed to load patch config from {}", patch_config_path.display()))?;
    println!("Patch config loaded: {:?}", patch_config);

    let current_crate_name = "split-decls-rs"; // This tool's crate name

    // Always generate a wrapped workspace in this mode (hardcoded for now)
    println!("Generating wrapped workspace in: {}", wrapped_workspace_output_dir.display());
    generate_wrapped_workspace(
        &wrapped_workspace_output_dir,
        &patch_config,
        &global_config,
        current_crate_name,
        dry_run,
    )?;
    println!("\nWrapped workspace generation finished.");

    println!("\nSplit-decls-rs tool finished.");
    Ok(())
}
