use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use toml::Value;
use tempfile::tempdir;
use crate::resolve_crate_path_in_submodule; // This will need to be resolved or removed if not needed within workspace-merge

/// Manages workspace dependencies in the root Cargo.toml.
pub fn manage_workspace_dependencies(root_cargo_toml_path: &Path, deps_to_add: &[(String, Value)], dry_run: bool) -> Result<()> {
    #[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
    struct Workspace {
        members: Option<Vec<String>>,
        #[serde(rename = "default-members")]
        default_members: Option<Vec<String>>,
        #[serde(default)]
        dependencies: HashMap<String, Value>, // Use HashMap for dependencies
        package: Option<toml::Table>,
        lints: Option<toml::Table>, // Added for workspace.lints
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct CargoToml {
        workspace: Option<Workspace>,
        patch: Option<toml::Table>,
        #[serde(flatten)]
        other: toml::Table,
    }

    let mut cargo_toml_content = fs::read_to_string(root_cargo_toml_path)
        .context(format!("Failed to read root Cargo.toml from {}", root_cargo_toml_path.display()))?;
    
    let mut cargo_toml: CargoToml = toml::from_str(&cargo_toml_content)
        .context(format!("Failed to parse root Cargo.toml from {}", root_cargo_toml_path.display()))?;

    let workspace_ref = cargo_toml.workspace.get_or_insert_with(Default::default);

    // Ensure workspace.package is initialized with defaults if None
    if workspace_ref.package.is_none() {
        workspace_ref.package = Some(get_default_workspace_package());
    }

    // Ensure workspace.lints is initialized as an empty table if None
    if workspace_ref.lints.is_none() {
        workspace_ref.lints = Some(toml::Table::new());
    }

    for (dep_name, dep_value) in deps_to_add {
        workspace_ref.dependencies.insert(dep_name.clone(), dep_value.clone());
    }

    // After handling workspace dependencies, now process patches
    let deps = &workspace_ref.dependencies; // Directly access the HashMap

    let patches = cargo_toml.patch.get_or_insert_with(toml::Table::new); // Get or create [patch]
    let crates_io_patches = patches.entry("crates-io".to_string())
                                   .or_insert_with(|| toml::Value::Table(toml::Table::new()))
                                   .as_table_mut()
                                   .context("crates-io patch must be a table")?;

    for (dep_name, dep_value) in deps.iter() {
        if let Some(dep_table) = dep_value.as_table() {
            if let Some(path_value) = dep_table.get("path") {
                if let Some(path_str) = path_value.as_str() {
                    let mut patch_entry = toml::Table::new();
                                                let resolved_path = crate::resolve_crate_path_in_submodule(&PathBuf::from(path_str), dep_name)?;
                                                patch_entry.insert("path".to_string(), toml::Value::String(resolved_path.to_str().context("Resolved path is not valid UTF-8")?.to_string()));
                    // If the workspace dependency has a 'package' key, also include it in the patch
                    if let Some(package_name) = dep_table.get("package") {
                        patch_entry.insert("package".to_string(), package_name.clone());
                    }
                    
                    crates_io_patches.insert(dep_name.clone(), toml::Value::Table(patch_entry));
                }
            }
        }
    }

    let new_cargo_toml_content = toml::to_string_pretty(&cargo_toml)
        .context("Failed to serialize new root Cargo.toml")?;
    
    if dry_run {
        let new_path = root_cargo_toml_path.with_extension("new");
        fs::write(&new_path, new_cargo_toml_content)
            .context(format!("Failed to write new root Cargo.toml to {}", new_path.display()))?;
        println!("Dry-run: Generated new root Cargo.toml content to {} for workspace.", new_path.display());
    } else {
        fs::write(root_cargo_toml_path, new_cargo_toml_content)
            .context(format!("Failed to write new root Cargo.toml to {}", root_cargo_toml_path.display()))?;
        println!("Successfully managed workspace dependencies in {}", root_cargo_toml_path.display());
    }

    Ok(())
}

pub fn apply_workspace_package_defaults_to_root(root_cargo_toml_path: &Path, dry_run: bool, verbose: bool) -> Result<()> {
    #[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
    struct Workspace {
        members: Option<Vec<String>>,
        #[serde(rename = "default-members")]
        default_members: Option<Vec<String>>,
        #[serde(default)]
        dependencies: HashMap<String, Value>,
        package: Option<toml::Table>,
        lints: Option<toml::Table>,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct CargoToml {
        workspace: Option<Workspace>,
        patch: Option<toml::Table>,
        #[serde(flatten)]
        other: toml::Table,
    }

    let cargo_toml_content = fs::read_to_string(root_cargo_toml_path)
        .context(format!("Failed to read root Cargo.toml from {}", root_cargo_toml_path.display()))?;
    
    let mut cargo_toml: CargoToml = toml::from_str(&cargo_toml_content)
        .context(format!("Failed to parse root Cargo.toml from {}", root_cargo_toml_path.display()))?;

    let mut workspace = cargo_toml.workspace.unwrap_or_else(Default::default);

    // Ensure workspace.package is initialized with defaults if None
    if workspace.package.is_none() {
        workspace.package = Some(get_default_workspace_package());
    }

    // Ensure workspace.lints is initialized as an empty table if None
    if workspace.lints.is_none() {
        workspace.lints = Some(toml::Table::new());
    }

    cargo_toml.workspace = Some(workspace); // Put the modified workspace back

    if verbose {
        println!("Debug: CargoToml before serialization: {:#?}", cargo_toml);
    }

    let new_cargo_toml_content = toml::to_string_pretty(&cargo_toml)
        .context("Failed to serialize new root Cargo.toml")?;
    
    if dry_run {
        let new_path = root_cargo_toml_path.with_extension("new");
        fs::write(&new_path, new_cargo_toml_content)
            .context(format!("Failed to write new root Cargo.toml to {}", new_path.display()))?;
        println!("Dry-run: Generated new root Cargo.toml content to {} for root package.", new_path.display());
    } else {
        fs::write(root_cargo_toml_path, new_cargo_toml_content)
            .context(format!("Failed to write new root Cargo.toml to {}", root_cargo_toml_path.display()))?;
        println!("Successfully applied workspace package defaults to root package in {}", root_cargo_toml_path.display());
    }

    Ok(())
}

fn get_default_workspace_package() -> toml::Table {
    let mut default_package = toml::Table::new();
    default_package.insert("edition".to_string(), toml::Value::String("2024".to_string()));
    default_package.insert("version".to_string(), toml::Value::String("1.0.0".to_string()));
    default_package.insert("publish".to_string(), toml::Value::Boolean(false));
    default_package.insert("keywords".to_string(), toml::Value::Array(vec![]));
    default_package.insert("rust-version".to_string(), toml::Value::String("1.85.0".to_string()));
    default_package.insert("include".to_string(), toml::Value::Array(vec![]));
    default_package.insert("license".to_string(), toml::Value::String("AGPL 3.0".to_string()));
    default_package.insert("authors".to_string(), toml::Value::Array(vec![]));
    default_package.insert("description".to_string(), toml::Value::String("".to_string()));
    default_package.insert("categories".to_string(), toml::Value::Array(vec![]));
    default_package.insert("repository".to_string(), toml::Value::String("".to_string()));
    default_package.insert("homepage".to_string(), toml::Value::String("".to_string()));
    default_package
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use tempfile::tempdir;

    #[test]
    fn test_manage_workspace_dependencies_with_package_section() -> Result<()> {
        let dir = tempdir()?;
        let root_cargo_toml_path = dir.path().join("Cargo.toml");

        let initial_content = r#" 
[workspace.package]
edition = "2021"
version = "0.1.0"
authors = ["Test Author"]

[workspace]
members = ["crate_a", "crate_b"]

[workspace.dependencies]
rand = "0.8"
"#;
        fs::write(&root_cargo_toml_path, initial_content)?;

        let deps_to_add = vec![
            ("serde".to_string(), toml::Value::String("1.0".to_string())),
        ];

        manage_workspace_dependencies(&root_cargo_toml_path, &deps_to_add, false)?;

        let modified_content = fs::read_to_string(&root_cargo_toml_path)?;
        println!("{}", modified_content);

        // Assert that the workspace.package section is preserved and dependencies are added.
        assert!(modified_content.contains("[workspace.package]"));
        assert!(modified_content.contains("edition = \"2021\""));
        assert!(modified_content.contains("version = \"0.1.0\""));
        assert!(modified_content.contains("authors = [\"Test Author\"]"));
        assert!(modified_content.contains("serde = \"1.0\""));
        assert!(modified_content.contains("rand = \"0.8\""));

        Ok(())
    }
}


    #[test]
    fn test_apply_workspace_package_defaults_to_root() -> Result<()> {
        let dir = tempdir()?;
        let root_cargo_toml_path = dir.path().join("Cargo.toml");

        let initial_content = r#" 
[workspace.package]
edition = "2024"
version = "1.0.0"
authors = ["Workspace Author"]
description = "Default workspace description"
homepage = "https://example.com"

[package]
name = "my-root-crate"
version = "0.9.0" # This should not be overwritten

[workspace]
members = ["crate_a"]
"#;
        fs::write(&root_cargo_toml_path, initial_content)?;

        apply_workspace_package_defaults_to_root(&root_cargo_toml_path, false, false)?;

        let modified_content = fs::read_to_string(&root_cargo_toml_path)?;
        println!("{}", modified_content);

        // Assert that values from workspace.package are applied to root package if not present
        assert!(modified_content.contains("name = \"my-root-crate\""));
        assert!(modified_content.contains("version = \"0.9.0\"")); // Should remain
        assert!(modified_content.contains("edition = \"2024\""));
        assert!(modified_content.contains("authors = [\"Workspace Author\"]"));
        assert!(modified_content.contains("description = \"Default workspace description\""));
        assert!(modified_content.contains("homepage = \"https://example.com\""));

        // Assert that workspace.package itself is still present
        assert!(modified_content.contains("[workspace.package]"));
        assert!(modified_content.contains("edition = \"2024\""));
        assert!(modified_content.contains("version = \"1.0.0\""));
        assert!(modified_content.contains("authors = [\"Workspace Author\"]"));

        Ok(())
    }

}
