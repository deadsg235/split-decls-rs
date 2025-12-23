use std::path::{Path, PathBuf};
pub mod macro_analyzer_parts;
pub mod special_print;
use anyhow::{Context, Result};

use split_decls_types::SplitDeclsConfig;

pub mod paths;

pub mod resolve_crate_path_in_submodule;
pub mod buildrs_ast_utils;
pub mod buildrs_generator;
pub mod git_manager;
pub mod patch_config;
pub mod workspace_manager;
pub mod eager_splitter;
pub mod extracted_decl;
pub mod generate_wrapped_workspace;
pub mod generate_wrapped_crate;
pub mod generate_new_cargotoml;
pub mod generate_new_lib_rs;
pub mod generate_new_build_rs;
pub mod apply_patches_to_syntax_tree;
pub mod auto_workspace_generator;
pub mod get_item_name;
pub mod get_item_kind;
pub mod process_crate;
pub mod generate_new_workspace;
pub mod crate_finder;
pub mod workflow_executor;
pub mod process_crates_in_path;
pub mod goal_parser;
pub mod backup_original_files;
pub mod backup_original_cargo;
pub mod ecosystem_processor;
pub use extracted_decl::*;

pub use crate::paths::{CratePaths, setup_crate_paths}; // Re-export from paths module

// Re-export key functions for tests
pub use process_crate::process_crate;
pub use process_crates_in_path::process_crates_in_path;
pub use generate_wrapped_workspace::generate_wrapped_workspace;

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
