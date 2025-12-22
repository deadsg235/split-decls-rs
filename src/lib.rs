use std::{
    fs,
    path::{Path, PathBuf}, // For running git commands
};
pub mod macro_analyzer_parts; // Declare the new module
pub mod special_print;
use anyhow::{Context, Result};
use quote::quote;
use walkdir::WalkDir;

use split_decls_types::SplitDeclsConfig;

pub mod resolve_crate_path_in_submodule;
pub mod buildrs_ast_utils;
pub mod buildrs_generator;
pub mod git_manager; // New module
pub mod patch_config; // New module
pub mod workspace_manager; // New module
pub mod eager_splitter;
pub mod extracted_decl;
pub mod generate_wrapped_workspace;
pub mod generate_wrapped_crate;
pub mod generate_new_cargotoml;
pub mod generate_new_lib_rs;
pub mod generate_new_build_rs;
pub mod apply_patches_to_syntax_tree;
pub mod get_item_name;
pub mod get_item_kind;
pub use extracted_decl::*;

use std::collections::HashMap;

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


// Helper to convert local path dependencies to workspace dependencies if they exist in global_config.workspace_dependencies
fn process_dependency_table(
    table: &mut toml::Table,
    global_config: &SplitDeclsConfig,
    original_crate_path: &Path, // New parameter
) -> Result<()> {
    let mut deps_to_update_to_workspace = Vec::new();
    let mut deps_to_update_to_absolute_path: Vec<(String, PathBuf)> = Vec::new();

    for (dep_name, dep_value) in table.iter_mut() {
        if let Some(dep_table) = dep_value.as_table_mut() {
            if let Some(path_value) = dep_table.get("path") {
                if let Some(path_str) = path_value.as_str() {
                    // Resolve the dependency path relative to the original crate's Cargo.toml
                    let resolved_original_dep_path = original_crate_path.join(path_str);

                    // Check if this path dependency corresponds to a workspace dependency
                    if global_config.workspace_dependencies.contains_key(dep_name) {
                        deps_to_update_to_workspace.push(dep_name.clone());
                    } else {
                        // If not a workspace dependency, convert to an absolute path
                        let absolute_path = resolved_original_dep_path.canonicalize()
                            .context(format!("Failed to canonicalize path for dependency '{}': {}", dep_name, resolved_original_dep_path.display()))?;
                        deps_to_update_to_absolute_path.push((dep_name.clone(), absolute_path));
                    }
                }
            }
        }
    }

    // First, update to workspace = true
    for dep_name in deps_to_update_to_workspace {
        let mut new_dep_table = toml::Table::new();
        new_dep_table.insert("workspace".to_string(), toml::Value::Boolean(true));
        // Preserve features if they exist in the original dependency
        if let Some(original_dep) = table.get(&dep_name) {
            if let Some(original_dep_table) = original_dep.as_table() {
                if let Some(features) = original_dep_table.get("features") {
                    new_dep_table.insert("features".to_string(), features.clone());
                }
            }
        }
        table.insert(dep_name, toml::Value::Table(new_dep_table));
    }

    // Then, update path dependencies to absolute paths
    for (dep_name, absolute_path) in deps_to_update_to_absolute_path {
        if let Some(dep_value) = table.get_mut(&dep_name) {
            if let Some(dep_table) = dep_value.as_table_mut() {
                dep_table.insert("path".to_string(), toml::Value::String(absolute_path.to_str().context("Path not valid UTF-8")?.to_string()));
            }
        }
    }
    Ok(())
}
