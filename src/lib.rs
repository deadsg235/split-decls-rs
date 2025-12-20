use std::{
    fs,
    path::{Path, PathBuf},
    process::Command, // For running git commands
};

use anyhow::{Context, Result};
use quote::quote;
use walkdir::WalkDir;
use toml; // Added for toml serialization

pub mod config;
use config::SplitDeclsConfig; // Import PatchSpec as well
pub mod buildrs_config;
pub mod buildrs_ast_utils;
pub mod buildrs_generator;
pub mod git_manager; // New module
pub mod patch_config; // New module
pub mod workspace_manager; // New module

#[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct Package {
        name: String,
        version: String,
        edition: String,
        #[serde(default)]
        workspace: Option<bool>, // To capture package.workspace = true
        // Add other fields from your Cargo.toml package section if needed
    }
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CargoToml {
    package: Package,
    lib: Option<toml::Table>,
    #[serde(default)]
    dependencies: toml::Table, // Changed from Option<toml::Table>
    #[serde(rename = "dev-dependencies")]
    #[serde(default)]
    dev_dependencies: toml::Table, // Changed from Option<toml::Table>
    #[serde(rename = "build-dependencies")]
    #[serde(default)]
    build_dependencies: toml::Table, // Changed from Option<toml::Table>
    #[serde(flatten)]
    #[serde(default)] // Ensure `other` is initialized even if empty
    other: toml::Table,
    #[serde(default)]
    patch: toml::Table, // New field for [patch] sections
}


/// Encapsulates all relevant file paths for a target crate.
pub struct CratePaths {
    pub crate_path: PathBuf,
    pub crate_name: String,
    pub lib_rs_path: PathBuf,
    pub old_lib_rs_path: PathBuf,
    pub build_rs_path: PathBuf,
    pub old_build_rs_path: PathBuf,
    pub cargo_toml_path: PathBuf, // New field for Cargo.toml
    pub old_cargo_toml_path: PathBuf, // New field for old Cargo.toml
    pub decls_output_dir: PathBuf,
    pub target_config_path: PathBuf,
}

/// Sets up and returns all relevant file paths for a given crate.
pub fn setup_crate_paths(crate_path: &Path) -> Result<CratePaths> {
    let crate_name_os_str = crate_path
        .file_name()
        .context("Crate path has no file name")?;
    let crate_name = crate_name_os_str
        .to_str()
        .context("Crate name is not valid UTF-8")?;

    let lib_rs_path = crate_path.join("src").join("lib.rs");
    let old_lib_rs_path = crate_path.join("src").join("oldlib.rs");
    let build_rs_path = crate_path.join("build.rs");
    let old_build_rs_path = crate_path.join("oldbuild.rs");
    let cargo_toml_path = crate_path.join("Cargo.toml");
    let old_cargo_toml_path = crate_path.join("oldCargo.toml"); // Define the path for the backed-up Cargo.toml
    let decls_output_dir = crate_path.join("src").join("decls");
    let target_config_path = crate_path.join(".split-decls-config.toml");

    Ok(CratePaths {
        crate_path: crate_path.to_path_buf(),
        crate_name: crate_name.to_string(),
        lib_rs_path,
        old_lib_rs_path,
        build_rs_path,
        old_build_rs_path,
        cargo_toml_path,
        old_cargo_toml_path,
        decls_output_dir,
        target_config_path,
    })
}


/// Generates the new Cargo.toml for the crate, adding necessary build-dependencies.
pub fn generate_new_cargotoml(paths: &CratePaths, global_config: &SplitDeclsConfig, dry_run: bool) -> Result<()> {
    // List of dependencies that should use `workspace = true`
    const RUNTIME_WORKSPACE_DEPS: &[&str] = &[
        "proc-macro2",
        "quote",
        "syn",
    ];

    const BUILD_WORKSPACE_DEPS: &[&str] = &[
        "anyhow",
        "proc-macro2",
        "quote",
        "syn",
        "introspector_decl2_macros",
        "introspector_decl_core", // Added as potentially needed for build scripts
        "introspector_macro_helpers", // Added as potentially needed for build scripts
        "introspector_decl_common", // Added as potentially needed for build scripts
    ];

    let mut cargo_toml_content = fs::read_to_string(&paths.old_cargo_toml_path)
        .context(format!("Failed to read old Cargo.toml from {}", paths.old_cargo_toml_path.display()))?;
    
    // If the file was empty (no Cargo.toml existed), initialize with a minimal structure
    if cargo_toml_content.trim().is_empty() {
        cargo_toml_content = format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            paths.crate_name
        );
    }

    let mut cargo_toml: CargoToml = toml::from_str(&cargo_toml_content)
        .context(format!("Failed to parse old Cargo.toml from {}", paths.old_cargo_toml_path.display()))?;

    // Remove unwanted top-level sections for submodules
    cargo_toml.other.remove("workspace");
    cargo_toml.other.remove("profile"); // This removes a top-level `[profile]` section
    cargo_toml.other.remove("lints"); // This removes a top-level `[lints]` section
    cargo_toml.other.remove("bench"); // Remove top-level `[bench]` sections

    // Iterate through `other` and remove any keys starting with "profile.", "lints.", "bench."
    // or containing "workspace" (unless it's specifically "package.workspace" which is handled in Package struct)
    let keys_to_remove: Vec<String> = cargo_toml.other.keys()
        .filter(|k| 
            k.starts_with("profile.") || 
            k.starts_with("lints.") || 
            k.starts_with("bench.") ||
            (k.contains("workspace") && *k != "workspace") // Remove other workspace-related keys
        )
        .cloned()
        .collect();

    for key in keys_to_remove {
        cargo_toml.other.remove(&key);
    }

    // Helper to ensure a dependency uses workspace = true
    let ensure_workspace_dependency = |table: &mut toml::Table, dep_name: &str| {
        let mut dep_table_value = toml::Table::new();
        dep_table_value.insert("workspace".to_string(), toml::Value::Boolean(true));
        table.insert(dep_name.to_string(), toml::Value::Table(dep_table_value));
    };

    // Process [dependencies] to ensure RUNTIME_WORKSPACE_DEPS are present
    let deps_table = &mut cargo_toml.dependencies;
    for dep_name in RUNTIME_WORKSPACE_DEPS {
        ensure_workspace_dependency(deps_table, dep_name);
    }

    // Process [dev-dependencies] (existing logic remains)
    let dev_deps_table = &mut cargo_toml.dev_dependencies;
    for dep_name in RUNTIME_WORKSPACE_DEPS { // Also update if runtime deps exist in dev-deps
        if dev_deps_table.contains_key(*dep_name) {
            ensure_workspace_dependency(dev_deps_table, dep_name);
        }
    }

    // Process [build-dependencies] to ensure BUILD_WORKSPACE_DEPS are present
    let build_deps_table = &mut cargo_toml.build_dependencies;
    for dep_name in BUILD_WORKSPACE_DEPS {
        ensure_workspace_dependency(build_deps_table, dep_name);
    }

    // Apply [patch.crates-io] entries
    if !global_config.crates_io_patches.is_empty() {
        let mut crates_io_table = toml::Table::new();
        for (crate_name, path) in &global_config.crates_io_patches {
            crates_io_table.insert(
                crate_name.clone(),
                toml::Table::from_iter([(
                    "path".to_string(),
                    toml::Value::String(path.to_str().context("Path not valid UTF-8")?.to_string()),
                )])
                .into(),
            );
        }
        cargo_toml.patch.insert("crates-io".to_string(), crates_io_table.into());
    }

    let new_cargo_toml_content = toml::to_string(&cargo_toml)
        .context("Failed to serialize new Cargo.toml")?;
    
    if dry_run {
        let new_path = paths.cargo_toml_path.with_extension("new"); // Changed to .new
        fs::write(&new_path, new_cargo_toml_content)
            .context(format!("Failed to write new Cargo.toml to {}", new_path.display()))?;
        println!("Dry-run: Generated new Cargo.toml content to {} for crate {}", new_path.display(), paths.crate_name);
    } else {
        fs::write(&paths.cargo_toml_path, new_cargo_toml_content)
            .context(format!("Failed to write new Cargo.toml to {}", paths.cargo_toml_path.display()))?;
        println!("Generated new Cargo.toml for crate {}", paths.crate_name);
    }

    Ok(())
}

/// Backs up original Cargo.toml file.
pub fn backup_original_cargotoml(paths: &CratePaths, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Dry-run: Would have backed up {} to {}", paths.cargo_toml_path.display(), paths.old_cargo_toml_path.display());
        return Ok(());
    }
    // Only perform the backup if the target Cargo.toml exists and a backup doesn't already exist.
    if paths.cargo_toml_path.exists() && !paths.old_cargo_toml_path.exists() {
        fs::rename(&paths.cargo_toml_path, &paths.old_cargo_toml_path)
            .context(format!("Failed to rename {} to {}", paths.cargo_toml_path.display(), paths.old_cargo_toml_path.display()))?;
        println!("Renamed {} to {}", paths.cargo_toml_path.display(), paths.old_cargo_toml_path.display());
    } else if paths.old_cargo_toml_path.exists() {
        println!("Backup {} already exists, skipping rename of {}", paths.old_cargo_toml_path.display(), paths.cargo_toml_path.display());
    } else if !paths.cargo_toml_path.exists() {
        // If no Cargo.toml exists and no backup exists, create an empty oldCargo.toml
        fs::write(&paths.old_cargo_toml_path, "")?;
        println!("No {} found, created empty {}", paths.cargo_toml_path.display(), paths.old_cargo_toml_path.display());
    }
    Ok(())
}

/// Backs up original lib.rs and build.rs files.
pub fn backup_original_files(paths: &CratePaths, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Dry-run: Would have backed up {} to {} and {} to {}", paths.lib_rs_path.display(), paths.old_lib_rs_path.display(), paths.build_rs_path.display(), paths.old_build_rs_path.display());
        return Ok(());
    }
    // Backup lib.rs
    if paths.lib_rs_path.exists() && !paths.old_lib_rs_path.exists() {
        fs::rename(&paths.lib_rs_path, &paths.old_lib_rs_path)
            .context(format!("Failed to rename {} to {}", paths.lib_rs_path.display(), paths.old_lib_rs_path.display()))?;
        println!("Renamed {} to {}", paths.lib_rs_path.display(), paths.old_lib_rs_path.display());
    } else if paths.old_lib_rs_path.exists() {
        println!("Backup {} already exists, skipping rename of {}", paths.old_lib_rs_path.display(), paths.lib_rs_path.display());
    } else if !paths.lib_rs_path.exists() {
        // If neither lib.rs nor oldlib.rs exists, create an empty oldlib.rs
        fs::write(&paths.old_lib_rs_path, "")?;
        println!("No {} found, created empty {}", paths.lib_rs_path.display(), paths.old_lib_rs_path.display());
    }
    
    // Backup build.rs
    if paths.build_rs_path.exists() && !paths.old_build_rs_path.exists() {
        fs::rename(&paths.build_rs_path, &paths.old_build_rs_path)
            .context(format!("Failed to rename {} to {}", paths.build_rs_path.display(), paths.old_build_rs_path.display()))?;
        println!("Renamed {} to {}", paths.build_rs_path.display(), paths.old_build_rs_path.display());
    } else if paths.old_build_rs_path.exists() {
        println!("Backup {} already exists, skipping rename of {}", paths.old_build_rs_path.display(), paths.build_rs_path.display());
    } else if !paths.build_rs_path.exists() {
        // If neither build.rs nor oldbuild.rs exists, create an empty oldbuild.rs
        fs::write(&paths.old_build_rs_path, "")?;
        println!("No {} found, created empty {}", paths.build_rs_path.display(), paths.old_build_rs_path.display());
    }
    Ok(())
}

/// Generates the new, minimal src/lib.rs for the crate.
pub fn generate_new_lib_rs(paths: &CratePaths, dry_run: bool) -> Result<()> {
    // 2. Generate new src/lib.rs for the crate
    // This new lib.rs will simply re-export items from the 'decls' module
    // and potentially other macros. The actual declarations will be in src/decls/*.rs
    let new_lib_rs_content = quote! {
        // Re-export prelude macros if desired
        pub use introspector_decl2_macros::prelude::*;

        // Include the module generated by the build.rs
        pub mod decls;
        pub use decls::*;
    };

    if dry_run {
        let new_path = paths.lib_rs_path.with_extension("new");
        fs::write(&new_path, new_lib_rs_content.to_string())
            .context(format!("Failed to write new lib.rs to {}", new_path.display()))?;
        println!("Dry-run: Generated new src/lib.rs content to {} for crate {}", new_path.display(), paths.crate_name);
    } else {
        fs::write(&paths.lib_rs_path, new_lib_rs_content.to_string())
            .context(format!("Failed to write new lib.rs to {}", paths.lib_rs_path.display()))?;
        println!("Generated new src/lib.rs for crate {}", paths.crate_name);
    }
    Ok(())
}

/// Generates the new build.rs for the crate.
pub fn generate_new_build_rs(paths: &CratePaths, dry_run: bool) -> Result<()> {
    // Generate the TokenStream for the build.rs content using the generator module
    let build_rs_token_stream = buildrs_generator::generate_build_rs_token_stream(
        &paths.old_lib_rs_path,
        &paths.old_build_rs_path,
        &paths.decls_output_dir,
        &paths.crate_name.replace("-", "_"),
    )?;

    if dry_run {
        let new_path = paths.build_rs_path.with_extension("new");
        fs::write(&new_path, build_rs_token_stream.to_string())
            .context(format!("Failed to write new build.rs to {}", new_path.display()))?;
        println!("Dry-run: Generated new build.rs content to {} for crate {}", new_path.display(), paths.crate_name);
    } else {
        fs::write(&paths.build_rs_path, build_rs_token_stream.to_string())
            .context(format!("Failed to write new build.rs to {}", paths.build_rs_path.display()))?;
        println!("Generated build.rs for crate {}", paths.crate_name);
    }
    Ok(())
}



pub fn process_crate(crate_path: &Path, global_config: &SplitDeclsConfig, dry_run: bool) -> Result<()> {
    let paths = setup_crate_paths(crate_path)?;
    println!("\n=== Processing crate: {} ===", paths.crate_name);

    // Create a crate-specific config for writing to .split-decls-config.toml
    let mut crate_config = SplitDeclsConfig::default();
    crate_config.active_overlay_modules = global_config.active_overlay_modules.clone();
    crate_config.custom_prelude_overlay = global_config.custom_prelude_overlay.clone();
    crate_config.string_replacements = global_config.string_replacements.clone();
    crate_config.crates_io_patches = global_config.crates_io_patches.clone(); // Copy new field

    // Filter patches relevant to this crate from the global config
    let crate_name_str = paths.crate_name.clone();
    if let Some(crate_patches) = global_config.patches.get(&crate_name_str) {
        crate_config.patches.insert(crate_name_str.clone(), crate_patches.clone());
    }
    // Note: rustc_source_path is not copied here, as it's a global setting for the tool,
    // not something needed by the generated build.rs

    let serialized_config = toml::to_string(&crate_config)
        .context("Failed to serialize SplitDeclsConfig")?;
    // Write the crate-specific config always, as it's an internal file for the build.rs
    fs::write(&paths.target_config_path, serialized_config)
        .context(format!("Failed to write .split-decls-config.toml to {}", paths.target_config_path.display()))?;
    println!("Wrote config to {}", paths.target_config_path.display());

    backup_original_cargotoml(&paths, dry_run)?;
    backup_original_files(&paths, dry_run)?;
    generate_new_cargotoml(&paths, global_config, dry_run)?;
    generate_new_lib_rs(&paths, dry_run)?;
    generate_new_build_rs(&paths, dry_run)?;

    Ok(())
}

pub fn process_crates_in_path(
    root_path: &Path,
    current_crate_name: &str,
    global_config: &SplitDeclsConfig,
    is_rustc_source: bool, // New parameter to control skipping logic
    dry_run: bool,
) -> Result<()> {
    for entry in WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok()) {
        if entry.file_name() == "Cargo.toml" {
            let cargotoml_path = entry.path();
            let crate_path = cargotoml_path
                .parent()
                .context("Cargo.toml has no parent directory")?;

            let crate_name = crate_path
                .file_name()
                .and_then(|s| s.to_str())
                .context("Could not get crate name")?;

            // Always skip self crate
            if crate_name == current_crate_name {
                println!("Skipping self crate: {}", current_crate_name);
                continue;
            }

            if !is_rustc_source {
                // Apply original skipping logic only for non-rustc source
                if crate_path.starts_with(Path::new("submodules/rust")) {
                    println!("Skipping rust submodule crate: {}", crate_name);
                    continue;
                }
                if crate_path.starts_with(Path::new("submodules/rust-analyzer/")) {
                    println!("Skipping rust-analyzer crate: {}", crate_name);
                    continue;
                }
                if crate_path.starts_with(Path::new("crates/trait-fixer")) {
                    println!("Skipping trait-fixer crate: {}", crate_name);
                    continue;
                }
                if crate_path.starts_with(Path::new("tools")) {
                    println!("Skipping tools crate: {}", crate_name);
                    continue;
                }
            } else {
                // For rustc source, skip the top-level virtual manifest if it exists
                if cargotoml_path == root_path.join("Cargo.toml") {
                    println!("Skipping root Cargo.toml of rustc source: {}", cargotoml_path.display());
                    continue;
                }
            }

            process_crate(crate_path, global_config, dry_run)?;
        }
    }
    Ok(())
}

// Helper function for recursive directory copy - this was added in the test, so move it to lib
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst).context(format!("Failed to create destination directory {}", dst.display()))?;
    for entry in fs::read_dir(src).context(format!("Failed to read source directory {}", src.display()))? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(&entry.path(), &dst.join(entry.file_name()))
                .context(format!("Failed to copy file from {} to {}", entry.path().display(), dst.join(entry.file_name()).display()))?;
        }
    }
    Ok(())
}
