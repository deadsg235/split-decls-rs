use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

use crate::patch_config;
use split_decls_types::SplitDeclsConfig;

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

fn should_skip_dir(path: &Path) -> bool {
    let name = path.file_name().unwrap().to_string_lossy();
    matches!(name.as_ref(), "target" | ".git" | "node_modules" | ".cargo")
}

struct SimpleCrateInfo {
    name: String,
}

fn extract_crate_info_simple(cargo_path: &Path) -> Result<Option<SimpleCrateInfo>> {
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
fn path_diff(from: &Path, to: &Path) -> Option<PathBuf> {
    path_relative_from(to, from)
}

fn path_relative_from(path: &Path, base: &Path) -> Option<PathBuf> {
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
    }
    
    if !dry_run {
        fs::create_dir_all(output_dir)
            .context(format!("Failed to create wrapped workspace directory: {}", output_dir.display()))?;
    }
    if verbose {
        println!("Wrapped workspace directory: {}", output_dir.display());
    }

    let mut workspace_members_content = Vec::new();
    let mut workspace_dependencies_content = String::from("introspector_decl2_macros = { path = \"introspector_decl2_macros\" }\n");
    let mut patch_crates_io_content = String::new();

    // Auto-generate workspace deps from project root using workspace manager
    let mut deps_to_add = Vec::new();
    
    if verbose {
        println!("Calling find_all_cargo_tomls in scan root: {}", scan_root.display());
    }
    // Find all Cargo.toml files and extract crate info
    if let Ok(cargo_tomls) = find_all_cargo_tomls(scan_root, verbose) {
        if verbose {
            println!("Found {} Cargo.toml files in scan root", cargo_tomls.len());
        }
        
        for cargo_path in cargo_tomls.iter() { // Removed .take(20) limit
            if verbose {
                println!("  Extracting crate info from: {}", cargo_path.display());
            }
            if let Ok(Some(crate_info)) = extract_crate_info_simple(&cargo_path) {
                let relative_path = cargo_path.parent().unwrap()
                    .strip_prefix(scan_root)
                    .unwrap_or(Path::new("."))
                    .to_string_lossy();
                
                let mut dep_table = toml::Table::new();
                dep_table.insert("path".to_string(), toml::Value::String(relative_path.to_string()));
                
                deps_to_add.push((crate_info.name.clone(), toml::Value::Table(dep_table)));
                workspace_members_content.push(format!("\"{}\"", relative_path));
                if verbose {
                    println!("    Added crate {} as workspace member and dependency candidate.", crate_info.name);
                }
            }
        }
        
        if verbose {
            println!("Generated {} workspace dependencies", deps_to_add.len());
        }
    } else {
        if verbose {
            println!("Failed to find any Cargo.toml files in scan root: {}", scan_root.display());
        }
    }

    // --- 1. Generate [workspace.members] ---
    for member in &patch_config.generated_workspace_member {
        if verbose {
            println!("Processing generated workspace member: {}", member.name);
        }
        workspace_members_content.push(format!("\"{}\"", member.path.display()));
        
        // Call generate_wrapped_crate for each member
        let original_crate_location = scan_root.join(&member.path); // Use scan_root as base
        
        if verbose {
            println!("  Calling generate_wrapped_crate for member '{}' at '{}'", member.name, original_crate_location.display());
        }
        generate_wrapped_crate::generate_wrapped_crate(
            output_dir,
            &member.name, // The original name of the crate, e.g., "unimacro_derive"
            &original_crate_location, // Path to the original crate's directory
            global_config,
            patch_config, // Pass patch_config here
            dry_run,
        )?;
        if verbose {
            println!("  Finished generate_wrapped_crate for member: {}", member.name);
        }
    }

    // --- 2. Generate [workspace.dependencies] and [patch.crates-io] ---
    
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
            // Calculate relative path from output_dir to the dependency's project_root_path
            let full_dep_path = scan_root.join(project_root_path); // Use scan_root as base
            let relative_path = path_diff(output_dir, &full_dep_path)
                .context(format!("Failed to calculate relative path from {} to {}", output_dir.display(), full_dep_path.display()))?;
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
            patch_crates_io_content.push_str(&dep_string);
        } else {
            workspace_dependencies_content.push_str(&dep_string);
        }
    }

    let mut workspace_cargo_toml_content = format!(
        r#"[workspace]
resolver = "2"
members = [
    {}
]

[workspace.dependencies]
{}"#,
        workspace_members_content.join(",\n    "),
        workspace_dependencies_content
    );

    if !patch_crates_io_content.is_empty() {
        workspace_cargo_toml_content.push_str("\n[patch.crates-io]\n");
        workspace_cargo_toml_content.push_str(&patch_crates_io_content);
    }
    
    let workspace_cargo_toml_path = output_dir.join("Cargo.toml");
    if !dry_run {
        fs::write(&workspace_cargo_toml_path, workspace_cargo_toml_content)
            .context(format!("Failed to write Cargo.toml for wrapped workspace: {}", workspace_cargo_toml_path.display()))?;
    }
    if verbose {
        println!("Generated workspace Cargo.toml at: {}", workspace_cargo_toml_path.display());
        println!("DEBUG: Exiting generate_wrapped_workspace.");
    }

    Ok(())
}