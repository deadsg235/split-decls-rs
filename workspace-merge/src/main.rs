use clap::Parser;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use std::fs;
use std::collections::HashMap;

use cargo_toml_generator_types::{CargoToml, Package, Workspace, Dependency, DependencyTable, PatchSection}; // Import necessary types

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

// read_cargo_toml function now uses the common CargoToml struct
fn read_cargo_toml(path: &Path) -> Result<CargoToml> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read Cargo.toml from {:?}", path))?;
    let toml: CargoToml = toml::from_str(&content)
        .context(format!("Failed to parse Cargo.toml from {:?}", path))?;
    Ok(toml)
}

// write_cargo_toml function now uses the common CargoToml struct
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
    // Clear its workspace_dependencies as it's no longer a workspace
    child_toml.workspace_dependencies.clear();
    println!("\nChild TOML (After removing workspace and clearing workspace_dependencies):\n{:#?}", child_toml);

    // 2. Add child to parent's `members`
    let child_relative_path = child_crate_root_path
        .strip_prefix(parent_workspace_root_path)
        .context("Child crate root is not a descendant of parent workspace root")?
        .to_str()
        .context("Failed to convert child relative path to string")?
        .to_string();

    // Ensure parent_toml has a workspace section to add members to
    if parent_toml.workspace.is_none() {
        parent_toml.workspace = Some(Workspace::default());
    }
    
    // Access and modify members. Since Workspace.members is now Vec<String>, no get_or_insert_with needed.
    if let Some(workspace) = &mut parent_toml.workspace {
        if !workspace.members.contains(&child_relative_path) {
            workspace.members.push(child_relative_path.clone());
            println!("Added '{}' to parent workspace members.", child_relative_path);
        } else {
            println!("'{}' already exists in parent workspace members.", child_relative_path);
        }
    }

    println!("\nParent TOML (After adding child to members):\n{:#?}", parent_toml);

    // 3. Merge workspace dependencies
    // Iterate over child_toml's direct dependencies
    for (dep_name, child_dep) in child_toml.dependencies.iter_mut() {
        let is_workspace_dep_in_child = match child_dep {
            Dependency::Version(_) => false, // Simple version string is not workspace=true
            Dependency::Table(table) => table.workspace == Some(true),
        };

        if is_workspace_dep_in_child {
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
        } else {
            // Check if this child's dependency should become a workspace dependency
            if parent_toml.workspace_dependencies.contains_key(dep_name) {
                // If child has a regular dependency that is now in parent's workspace_dependencies,
                // update child to use workspace = true
                *child_dep = Dependency::Table(DependencyTable {
                    workspace: Some(true),
                    ..Default::default()
                });
                println!("Updated child dependency '{}' to use workspace = true as it's in parent's workspace.", dep_name);
            }
        }
    }
    // Iterate over child_toml's dev-dependencies
    for (dep_name, child_dep) in child_toml.dev_dependencies.iter_mut() {
        let is_workspace_dep_in_child = match child_dep {
            Dependency::Version(_) => false,
            Dependency::Table(table) => table.workspace == Some(true),
        };

        if is_workspace_dep_in_child {
            if !parent_toml.workspace_dependencies.contains_key(dep_name) {
                parent_toml.workspace_dependencies.insert(dep_name.clone(), child_dep.clone());
                println!("Migrated dev-dependency '{}' to parent's workspace dependencies.", dep_name);
            } else {
                println!("dev-dependency '{}' already exists in parent's workspace dependencies. Keeping parent's definition.", dep_name);
            }
        } else {
            if parent_toml.workspace_dependencies.contains_key(dep_name) {
                *child_dep = Dependency::Table(DependencyTable {
                    workspace: Some(true),
                    ..Default::default()
                });
                println!("Updated child dev-dependency '{}' to use workspace = true as it's in parent's workspace.", dep_name);
            }
        }
    }


    // Write back modified Cargo.toml files
    write_cargo_toml(&parent_cargo_toml_path, &parent_toml)?;
    write_cargo_toml(&child_crate_cargo_toml_path, &child_toml)?;

    println!("\nSuccessfully merged workspace.");
    Ok(())
}
