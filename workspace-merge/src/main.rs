use clap::Parser;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the parent workspace's Cargo.toml
    #[arg(short, long)]
    parent_workspace_root: PathBuf,

    /// Path to the child crate that should be merged into the parent workspace
    #[arg(short, long)]
    child_crate_root: PathBuf,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Package {
    name: String,
    version: String,
    edition: String,
    #[serde(default)]
    authors: Vec<String>, // Authors can be an empty array
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Dependency {
    // This could be more complex, but for now, just store the version string
    version: Option<String>,
    path: Option<String>,
    workspace: Option<bool>,
    // Add other fields as needed
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Workspace {
    members: Option<Vec<String>>,
    // Dependencies within a [workspace] section are typically handled by [workspace.dependencies] top-level table
    // Removing `dependencies` field from here.
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct CargoToml {
    package: Option<Package>,
    workspace: Option<Workspace>,
    #[serde(rename = "workspace.dependencies")]
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")] // Add this line
    workspace_dependencies: HashMap<String, Dependency>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    dependencies: HashMap<String, Dependency>,
    #[serde(flatten)] // Add this line
    other: toml::Table, // Add this field
}

fn read_cargo_toml(path: &Path) -> Result<CargoToml> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read Cargo.toml from {:?}", path))?;
    let toml: CargoToml = toml::from_str(&content)
        .context(format!("Failed to parse Cargo.toml from {:?}", path))?;
    Ok(toml)
}

fn write_cargo_toml(path: &Path, toml_data: &CargoToml) -> Result<()> {
    let toml_string = toml::to_string_pretty(toml_data)
        .context(format!("Failed to serialize Cargo.toml data for {:?}", path))?;
    fs::write(path, toml_string)
        .context(format!("Failed to write Cargo.toml to {:?}", path))?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    let parent_workspace_root_path = &args.parent_workspace_root;
    let child_crate_root_path = &args.child_crate_root;

    let parent_cargo_toml_path = parent_workspace_root_path.join("Cargo.toml");
    let child_crate_cargo_toml_path = child_crate_root_path.join("Cargo.toml");

    println!("Parent Cargo.toml path: {:?}", parent_cargo_toml_path);
    println!("Child Crate Cargo.toml path: {:?}", child_crate_cargo_toml_path);

    let mut parent_toml = read_cargo_toml(&parent_cargo_toml_path)?;
    let mut child_toml = read_cargo_toml(&child_crate_cargo_toml_path)?;

    println!("\nParent TOML (Before):\n{:#?}", parent_toml);
    println!("\nChild TOML (Before):\n{:#?}", child_toml);

    // 1. Remove [workspace] from the child's Cargo.toml
    child_toml.workspace = None;
    // CRITICAL FIX: Also clear its workspace_dependencies as it's no longer a workspace
    child_toml.workspace_dependencies.clear();
    println!("\nChild TOML (After removing workspace and clearing workspace_dependencies):\n{:#?}", child_toml);

    // 2. Add child to parent's `members`
    let child_relative_path = child_crate_root_path
        .strip_prefix(parent_workspace_root_path)
        .context("Child crate root is not a descendant of parent workspace root")?
        .to_str()
        .context("Failed to convert child relative path to string")?
        .to_string();

    if let Some(workspace) = &mut parent_toml.workspace {
        let members = workspace.members.get_or_insert_with(Vec::new);
        if !members.contains(&child_relative_path) {
            members.push(child_relative_path.clone());
            println!("Added '{}' to parent workspace members.", child_relative_path);
        } else {
            println!("'{}' already exists in parent workspace members.", child_relative_path);
        }
    } else {
        // If parent_toml has no workspace section, create one
        parent_toml.workspace = Some(Workspace {
            members: Some(vec![child_relative_path.clone()]),
            ..Default::default()
        });
        println!("Created parent workspace and added '{}' to members.", child_relative_path);
    }

    println!("\nParent TOML (After adding child to members):\n{:#?}", parent_toml);

    // 3. Merge workspace dependencies
    for (dep_name, child_dep) in child_toml.dependencies.iter_mut() {
        if child_dep.workspace == Some(true) {
            // If child's dependency explicitly uses workspace = true, ensure it's in parent's workspace_dependencies
            if !parent_toml.workspace_dependencies.contains_key(dep_name) {
                // Add to parent's workspace_dependencies
                parent_toml.workspace_dependencies.insert(dep_name.clone(), child_dep.clone());
                println!("Migrated '{}' to parent's workspace dependencies.", dep_name);
            } else {
                // Dependency already exists in parent's workspace_dependencies.
                // For simplicity, we assume compatibility or parent's definition takes precedence.
                println!("'{}' already exists in parent's workspace dependencies. Keeping parent's definition.", dep_name);
            }
        } else if parent_toml.workspace_dependencies.contains_key(dep_name) {
            // If child has a regular dependency that is now in parent's workspace_dependencies,
            // update child to use workspace = true
            child_dep.version = None; // Remove local version
            child_dep.path = None; // Remove local path
            child_dep.workspace = Some(true); // Set to workspace = true
            println!("Updated child dependency '{}' to use workspace = true as it's in parent's workspace.", dep_name);
        }
    }

    // Write back modified Cargo.toml files
    write_cargo_toml(&parent_cargo_toml_path, &parent_toml)?;
    write_cargo_toml(&child_crate_cargo_toml_path, &child_toml)?;

    println!("\nSuccessfully merged workspace.");
    Ok(())
}
