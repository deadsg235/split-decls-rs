use anyhow::{Context, Result};
use std::collections::HashMap; // Add this import
use std::fs;
use std::path::{Path, PathBuf};
use toml::{Table, Value}; // Import Table
use chrono::Utc;

use crate::add_generated_header;

use crate::patch_config;
use split_decls_types::SplitDeclsConfig;
use crate::process_dependencies_for_output_crate;
use crate::generate_wrapped_crate;

fn find_all_cargo_tomls(dir: &Path, verbose: bool) -> Result<Vec<PathBuf>> {
    let mut cargo_tomls = Vec::new();
    
    if dir.is_dir() {
        if verbose {
            println!("  Scanning directory: {}", dir.display());
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.file_name() == Some("Cargo.toml".as_ref()) {
                if verbose {
                    println!("    Found Cargo.toml: {}", path.display());
                }
                cargo_tomls.push(path);
            } 
            // Removed the recursive call to find_all_cargo_tomls to only scan the immediate directory
            // and removed the should_skip_dir check as it's not recursive anymore.
        }
    }
    
    Ok(cargo_tomls)
}

pub fn should_skip_dir(path: &Path) -> bool {
    let name = path.file_name().unwrap().to_string_lossy();
    matches!(name.as_ref(), "target" | ".git" | "node_modules" | ".cargo")
}

struct SimpleCrateInfo {
    name: String,
}

pub fn extract_crate_info_simple(cargo_path: &Path) -> Result<Option<SimpleCrateInfo>> {
    let content = fs::read_to_string(cargo_path)?;
    let toml: Value = toml::from_str(&content)?;
    
    if let Some(package) = toml.get("package") {
        if let Some(name) = package.get("name").and_then(|n| n.as_str()) {
            return Ok(Some(SimpleCrateInfo {
                name: name.to_string(),
            }));
        }
    }
    
    Ok(None)
}

/// Calculates the relative path from one directory to another.
pub fn path_diff(from: &Path, to: &Path) -> Option<PathBuf> {
    path_relative_from(to, from)
}

pub fn path_relative_from(path: &Path, base: &Path) -> Option<PathBuf> {
    let mut relativized_path = PathBuf::new();
    let mut common_prefix = 0;

    for (p_comp, b_comp) in path.components().zip(base.components()) {
        if p_comp == b_comp {
            common_prefix += 1;
        } else {
            break;
        }
    }

    let num_up = base.components().count() - common_prefix;
    for _ in 0..num_up {
        relativized_path.push("..");
    }

    for p_comp in path.components().skip(common_prefix) {
        relativized_path.push(p_comp);
    }

    if relativized_path.as_os_str().is_empty() {
        Some(PathBuf::from("."))
    } else {
        Some(relativized_path)
    }
}


/// Generates a new workspace containing "wrapped" versions of the target crates.
/// Each wrapped crate will have its declarations eagerly split and patched.
pub fn generate_wrapped_workspace(
    output_dir: &Path,
    patch_config: &patch_config::PatchConfig,
    global_config: &SplitDeclsConfig,
    scan_root: &Path,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("DEBUG: generate_wrapped_workspace called with output_dir: {}", output_dir.display());
        println!("DEBUG: Scanning root for Cargo.tomls: {}", scan_root.display());
        println!("DEBUG: patch_config.generated_workspace_member.is_empty(): {}", patch_config.generated_workspace_member.is_empty());
    }
    
    if !dry_run {
        fs::create_dir_all(output_dir)
            .context(format!("Failed to create wrapped workspace directory: {}", output_dir.display()))?;
    }
    if verbose {
        println!("Wrapped workspace directory: {}", output_dir.display());
    }

    let mut final_cargo_toml_content = String::new();
    let main_crate_cargo_toml_path = scan_root.join("Cargo.toml");

    // Scenario: Wrapping a single crate (e.g., in bootstrap mode for split-decls-rs itself)
    // This is indicated if patch_config.generated_workspace_member is empty,
    // and scan_root is presumed to be the single crate.
    if patch_config.generated_workspace_member.is_empty() && main_crate_cargo_toml_path.exists() {
        if verbose {
            println!("DEBUG: Detected single crate wrapping scenario. Generating [package] Cargo.toml.");
        }
        let root_cargo_toml_content = fs::read_to_string(&main_crate_cargo_toml_path)
            .context(format!("Failed to read Cargo.toml from scan_root: {}", main_crate_cargo_toml_path.display()))?;
        let root_cargo_toml: Table = toml::from_str(&root_cargo_toml_content)
            .context(format!("Failed to parse Cargo.toml from scan_root: {}", main_crate_cargo_toml_path.display()))?;

        // Construct [package] section
        if let Some(package_section) = root_cargo_toml.get("package").and_then(|v| v.as_table()) {
            // Add the workspace declaration first for a single crate being wrapped as its own workspace
            final_cargo_toml_content.push_str("[workspace]\n\n");
            final_cargo_toml_content.push_str("[package]\n");
            for (key, value) in package_section.iter() {
                // Skip workspace-related keys if they exist in the package section (unlikely but good practice)
                if key != "workspace" {
                    final_cargo_toml_content.push_str(&format!("{} = {}\n", key, value.to_string()));
                }
            }
        } else {
            anyhow::bail!("No [package] section found in {}", main_crate_cargo_toml_path.display());
        }

        // Process and construct [dependencies]
        if let Some(deps) = root_cargo_toml.get("dependencies").and_then(|v| v.as_table()) {
            let processed_deps = process_dependencies_for_output_crate(deps, global_config, output_dir, scan_root, verbose)?;
            if !processed_deps.is_empty() {
                final_cargo_toml_content.push_str("\n[dependencies]\n");
                for (key, value) in processed_deps.iter() {
                    final_cargo_toml_content.push_str(&format!("{} = {}\n", key, value.to_string()));
                }
            }
        }

        // Process and construct [build-dependencies]
        if let Some(build_deps) = root_cargo_toml.get("build-dependencies").and_then(|v| v.as_table()) {
            let processed_build_deps = process_dependencies_for_output_crate(build_deps, global_config, output_dir, scan_root, verbose)?;
            if !processed_build_deps.is_empty() {
                final_cargo_toml_content.push_str("\n[build-dependencies]\n");
                for (key, value) in processed_build_deps.iter() {
                    final_cargo_toml_content.push_str(&format!("{} = {}\n", key, value.to_string()));
                }
            }
        }

        // Process and construct [dev-dependencies]
        if let Some(dev_deps) = root_cargo_toml.get("dev-dependencies").and_then(|v| v.as_table()) {
            let processed_dev_deps = process_dependencies_for_output_crate(dev_deps, global_config, output_dir, scan_root, verbose)?;
            if !processed_dev_deps.is_empty() {
                final_cargo_toml_content.push_str("\n[dev-dependencies]\n");
                for (key, value) in processed_dev_deps.iter() {
                    final_cargo_toml_content.push_str(&format!("{} = {}\n", key, value.to_string()));
                }
            }
        }

        // Handle [[bin]] sections - direct copy of relevant parts, but adjust paths
        if let Some(bin_array) = root_cargo_toml.get("bin").and_then(|v| v.as_array()) {
            for bin_item in bin_array {
                if let Some(bin_table) = bin_item.as_table() {
                    final_cargo_toml_content.push_str("\n[[bin]]\n");
                    for (key, value) in bin_table.iter() {
                        if key == "path" {
                            if let Some(path_str) = value.as_str() {
                                let absolute_bin_path = scan_root.join(path_str);
                                let relative_bin_path = path_diff(output_dir, &absolute_bin_path)
                                    .context(format!("Failed to calculate relative path for bin '{}'", path_str))?;
                                final_cargo_toml_content.push_str(&format!("path = \"{}\"\n", relative_bin_path.display()));
                            }
                        } else {
                            final_cargo_toml_content.push_str(&format!("{} = {}\n", key, value.to_string()));
                        }
                    }
                }
            }
        }

        // Handle [patch] sections
        if let Some(patch_section) = root_cargo_toml.get("patch").and_then(|v| v.as_table()) {
            if !patch_section.is_empty() {
                final_cargo_toml_content.push_str("\n[patch.crates-io]\n");
                if let Some(crates_io_patch) = patch_section.get("crates-io").and_then(|v| v.as_table()) {
                    let processed_patch_deps = process_dependencies_for_output_crate(crates_io_patch, global_config, output_dir, scan_root, verbose)?;
                     for (key, value) in processed_patch_deps.iter() {
                        final_cargo_toml_content.push_str(&format!("{} = {}\n", key, value.to_string()));
                    }
                }
            }
        }

    } else {
        // Existing logic for workspace generation (if patch_config.generated_workspace_member is NOT empty)
        // This part remains mostly the same, but integrate the new dependency processing if applicable
        
        let mut workspace_members_content = Vec::new();
        let mut workspace_dependencies_content_str = String::new(); // Use a new name to avoid conflict
        let mut patch_crates_io_content_str = String::new(); // Use a new name to avoid conflict

        // Auto-generate workspace deps from project root using workspace manager
        // This block needs to be carefully considered if it's still relevant when generating a workspace
        // Currently, it populates `workspace_members_content` and `deps_to_add`
        if verbose {
            println!("Calling find_all_cargo_tomls in scan root: {}", scan_root.display());
        }
        if let Ok(cargo_tomls) = find_all_cargo_tomls(scan_root, verbose) {
            if verbose {
                println!("Found {} Cargo.toml files in scan root", cargo_tomls.len());
            }
            for cargo_path in cargo_tomls.iter() {
                if verbose {
                    println!("  Extracting crate info from: {}", cargo_path.display());
                }
                if let Ok(Some(crate_info)) = extract_crate_info_simple(&cargo_path) {
                    let relative_path = cargo_path.parent().unwrap()
                        .strip_prefix(scan_root)
                        .unwrap_or(Path::new("."))
                        .to_string_lossy();
                    
                    // No longer need to push to deps_to_add directly here as we're building the workspace TOML
                    workspace_members_content.push(format!("\"{}\"", relative_path));
                    if verbose {
                        println!("    Added crate {} as workspace member and dependency candidate.", crate_info.name);
                    }
                }
            }
            if verbose {
                println!("Generated {} workspace members from scan root", workspace_members_content.len());
            }
        } else {
            if verbose {
                println!("Failed to find any Cargo.toml files in scan root: {}", scan_root.display());
            }
        }

        // Process generated_workspace_member from patch_config
        for member in &patch_config.generated_workspace_member {
            if verbose {
                println!("Processing generated workspace member: {}", member.name);
            }
            workspace_members_content.push(format!("\"{}\"", member.path.display()));
            
            // Call generate_wrapped_crate for each member
            let original_crate_location = scan_root.join(&member.path);
            
            if verbose {
                println!("  Calling generate_wrapped_crate for member '{}' at '{}'", member.name, original_crate_location.display());
            }
            // This is where individual members are processed and their generated content written
            generate_wrapped_crate::generate_wrapped_crate(
                output_dir,
                &member.name,
                &original_crate_location,
                global_config,
                patch_config,
                dry_run,
            )?;
            if verbose {
                println!("  Finished generate_wrapped_crate for member: {}", member.name);
            }
        }
        
        // Process generated_workspace_dependency from patch_config
        for dep in &patch_config.generated_workspace_dependency {
            if verbose {
                println!("Processing generated workspace dependency: {}", dep.name);
            }
            let mut dep_string = format!("{} = {{ ", dep.name);

            let mut parts = Vec::new();

            if let Some(version) = &dep.version {
                parts.push(format!("version = \"{}\"", version));
            }

            if let Some(project_root_path) = &dep.project_root_path {
                let full_dep_path = scan_root.join(project_root_path);
                let relative_path = path_diff(output_dir, &full_dep_path)
                    .ok_or_else(|| anyhow::anyhow!(format!("Failed to calculate relative path for workspace dep '{}'", dep.name)))?;
                parts.push(format!("path = \"{}\"", relative_path.display()));
            }

            if let Some(features) = &dep.features {
                parts.push(format!("features = [\"{}\"]", features.join("\", \"")));
            }

            if let Some(package) = &dep.package {
                parts.push(format!("package = \"{}\"", package));
            }

            dep_string.push_str(&parts.join(", "));
            dep_string.push_str(" }\n");

            if dep.is_patch.unwrap_or(false) {
                patch_crates_io_content_str.push_str(&dep_string);
            } else {
                workspace_dependencies_content_str.push_str(&dep_string);
            }
        }

        // Construct final workspace Cargo.toml content
        final_cargo_toml_content = format!(
            r#"[workspace]
resolver = "2"
members = [
    {}
]

[workspace.dependencies]
introspector_decl2_macros = {{ path = "../../submodules/patch-build-rs/introspector_decl2_macros" }} # Explicitly keep this one if it's always needed
{}"#,
            workspace_members_content.join(",\n    "),
            workspace_dependencies_content_str
        );

        if !patch_crates_io_content_str.is_empty() {
            final_cargo_toml_content.push_str("\n[patch.crates-io]\n");
            final_cargo_toml_content.push_str(&patch_crates_io_content_str);
        }
    } // End of else (workspace generation)
    
            let workspace_cargo_toml_path = output_dir.join("Cargo.toml");
            if !dry_run {
                add_generated_header!(&workspace_cargo_toml_path, final_cargo_toml_content.as_str())
                    .context(format!("Failed to write Cargo.toml for single crate: {}", workspace_cargo_toml_path.display()))?;
            }    if verbose {
        println!("Generated Cargo.toml at: {}", workspace_cargo_toml_path.display());
        println!("DEBUG: Exiting generate_wrapped_workspace.");
    }

    Ok(())
}
