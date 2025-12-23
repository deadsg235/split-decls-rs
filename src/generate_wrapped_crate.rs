use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::patch_config;
use split_decls_types::SplitDeclsConfig;
use crate::setup_crate_paths;
use crate::generate_new_cargotoml::generate_new_cargotoml;
use crate::generate_new_lib_rs::generate_new_lib_rs;
use crate::generate_new_build_rs::generate_new_build_rs;
use crate::apply_patches_to_syntax_tree::apply_patches_to_syntax_tree;
use std::collections::HashMap;
use crate::eager_splitter;

/// Generates a new workspace containing "wrapped" versions of the target crates.
/// Each wrapped crate will have its declarations eagerly split and patched.
#[allow(clippy::too_many_arguments)] // This function has many arguments, but they are all necessary.
pub fn generate_wrapped_crate(
    wrapped_workspace_root: &Path,
    original_crate_name: &str, // The original name of the crate, e.g., "unimacro_derive"
    original_crate_path: &Path, // Path to the original crate's directory (relative to project root)
    global_config: &SplitDeclsConfig,
    patch_config: &patch_config::PatchConfig, // New: Pass patch_config here
    dry_run: bool,
) -> Result<()> {
    let wrapped_crate_name = format!("wrapped-{}", original_crate_name);
    println!("\n=== Generating wrapped crate: {} ===", wrapped_crate_name);

    let wrapped_crate_path = wrapped_workspace_root.join(&wrapped_crate_name);
    if !dry_run {
        fs::create_dir_all(&wrapped_crate_path)
            .context(format!("Failed to create wrapped crate directory: {}", wrapped_crate_path.display()))?;
        fs::create_dir_all(&wrapped_crate_path.join("src"))
            .context(format!("Failed to create src directory for wrapped crate: {}", wrapped_crate_path.display()))?;
    }

    // Setup CratePaths for the *wrapped* crate
    // The `old_lib_rs_path` etc. for this `CratePaths` will refer to the copies *within* the wrapped crate.
    let wrapped_crate_paths = setup_crate_paths(&wrapped_crate_path)?;

    // Copy original oldlib.rs and oldCargo.toml into the wrapped crate's location
    // These will be the source for eager splitting within the wrapped crate context.
    let original_lib_rs_path = original_crate_path.join("src").join("lib.rs");
    let original_cargo_toml_path = original_crate_path.join("Cargo.toml");
    
    if original_lib_rs_path.exists() {
        if !dry_run {
            fs::copy(&original_lib_rs_path, &wrapped_crate_paths.old_lib_rs_path)
                .context(format!("Failed to copy {} to {}", original_lib_rs_path.display(), wrapped_crate_paths.old_lib_rs_path.display()))?;
        }
        println!("Copied original lib.rs to {}", wrapped_crate_paths.old_lib_rs_path.display());
    } else {
        // If no original lib.rs, create an empty oldlib.rs in the wrapped crate
        if !dry_run { fs::write(&wrapped_crate_paths.old_lib_rs_path, "")?; }
        println!("No original lib.rs found, created empty {}", wrapped_crate_paths.old_lib_rs_path.display());
    }

    if original_cargo_toml_path.exists() {
        if !dry_run {
            fs::copy(&original_cargo_toml_path, &wrapped_crate_paths.old_cargo_toml_path)
                .context(format!("Failed to copy {} to {}", original_cargo_toml_path.display(), wrapped_crate_paths.old_cargo_toml_path.display()))?;
        }
        println!("Copied original Cargo.toml to {}", wrapped_crate_paths.old_cargo_toml_path.display());
    } else {
        // If no original Cargo.toml, create an empty oldCargo.toml in the wrapped crate
        if !dry_run { fs::write(&wrapped_crate_paths.old_cargo_toml_path, "")?; }
        println!("No original Cargo.toml found, created empty {}", wrapped_crate_paths.old_cargo_toml_path.display());
    }


    // Generate Cargo.toml for the wrapped crate
    // This will reference the original project's crates if they are part of the original workspace
    generate_new_cargotoml(
        &original_cargo_toml_path,
        &wrapped_crate_paths.cargo_toml_path, // Output path for the new Cargo.toml
        original_crate_path,
        global_config,
        patch_config,
        dry_run,
    )?;
    
    // Generate lib.rs for the wrapped crate
    generate_new_lib_rs(&wrapped_crate_paths, dry_run)?;

    // Create a crate-specific config for writing to .split-decls-config.toml
    let mut crate_config = SplitDeclsConfig::default();
    crate_config.active_overlay_modules = global_config.active_overlay_modules.clone();
    crate_config.custom_prelude_overlay = global_config.custom_prelude_overlay.clone();
    crate_config.string_replacements = global_config.string_replacements.clone();
    crate_config.crates_io_patches = global_config.crates_io_patches.clone();

    // Filter patches relevant to this crate from the global config
    let crate_name_str = original_crate_name.to_string(); // Use original crate name for patch lookup
    if let Some(global_patches_map) = &global_config.patches {
        if let Some(patches_for_crate) = global_patches_map.get(&crate_name_str) {
            let mut new_patches_map = HashMap::new();
            new_patches_map.insert(crate_name_str.clone(), patches_for_crate.clone());
            crate_config.patches = Some(new_patches_map);
        }
    }

    let serialized_config = toml::to_string(&crate_config)
        .context("Failed to serialize SplitDeclsConfig for wrapped crate")?;
    if !dry_run {
        fs::write(&wrapped_crate_paths.target_config_path, serialized_config)
            .context(format!("Failed to write .split-decls-config.toml to {}", wrapped_crate_paths.target_config_path.display()))?;
    }
    println!("Wrote config to {}", wrapped_crate_paths.target_config_path.display());


    // Eager Splitting Logic for the wrapped crate
    let mut old_lib_rs_content = fs::read_to_string(&wrapped_crate_paths.old_lib_rs_path)
        .context("Failed to read oldlib.rs content for eager splitting in wrapped crate")?;

    if let Some(replacements) = &global_config.string_replacements {
        for sr in replacements {
            old_lib_rs_content = old_lib_rs_content.replace(&sr.old, &sr.new);
            println!("Applied string replacement: '{}' -> '{}'", sr.old, sr.new);
        }
    }

    let mut syntax_tree = syn::parse_file(&old_lib_rs_content)
        .context("Failed to parse oldlib.rs content for eager splitting in wrapped crate")?;

    apply_patches_to_syntax_tree(
        &mut syntax_tree,
        &wrapped_crate_name.replace("-", "_"), // Use wrapped crate name for patching
        &crate_config,
    )?;

    eager_splitter::split_and_generate_decls(
        &syntax_tree,
        &wrapped_crate_paths,
        &crate_config,
        dry_run,
    )?;

    // Generate build.rs for the wrapped crate (minimal version for monitoring patches)
    generate_new_build_rs(&wrapped_crate_paths, dry_run)?;

    Ok(())
}