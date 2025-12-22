use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use crate::{CratePaths, CargoToml, process_dependency_table};
use split_decls_types::SplitDeclsConfig;
use crate::patch_config;

/// Generates the new Cargo.toml for the crate, adding necessary build-dependencies.
pub fn generate_new_cargotoml(
    paths: &CratePaths,
    global_config: &SplitDeclsConfig, // Still needed for root workspace info
    original_crate_real_path: &Path,
    patch_config: &patch_config::PatchConfig, // New: to get crate-specific dependencies
    dry_run: bool,
) -> Result<()> {
    let mut cargo_toml_content = fs::read_to_string(&paths.old_cargo_toml_path)
        .context(format!("Failed to read old Cargo.toml from {}", paths.old_cargo_toml_path.display()))?;
    
    // If the file was empty (no Cargo.toml existed), initialize with a minimal structure
    if cargo_toml_content.trim().is_empty() {
        cargo_toml_content = format!(
            "[package]\nname = \"{}\nversion = \"0.1.0\"\nedition = \"2021\"\n",
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

    // Process existing dependencies from the old Cargo.toml
    // This will convert any path dependencies to workspace = true if they are found in global_config.workspace_dependencies
    process_dependency_table(&mut cargo_toml.dependencies, global_config, original_crate_real_path)?;
    process_dependency_table(&mut cargo_toml.dev_dependencies, global_config, original_crate_real_path)?;
    process_dependency_table(&mut cargo_toml.build_dependencies, global_config, original_crate_real_path)?;


    // Ensure essential build dependencies are present
    let build_deps_table = &mut cargo_toml.build_dependencies;
    let essential_build_deps = [
        ("anyhow", None),
        ("syn", Some(vec!["full", "visit"])),
        ("serde", Some(vec!["derive"])),
        ("toml", None),
    ];

    for (dep_name, features) in essential_build_deps {
        let mut dep_table_value = toml::Table::new();
        dep_table_value.insert("workspace".to_string(), toml::Value::Boolean(true));
        if let Some(feats) = features {
            let features_array = toml::Value::Array(
                feats.into_iter().map(|f| toml::Value::String(f.to_string())).collect()
            );
            dep_table_value.insert("features".to_string(), features_array);
        }
        build_deps_table.insert(dep_name.to_string(), toml::Value::Table(dep_table_value));
    }

    // Add introspector macro crate as regular dependency
    let mut macro_dep = toml::Table::new();
    macro_dep.insert("path".to_string(), toml::Value::String("../../introspector_decl2_macros".to_string()));
    cargo_toml.dependencies.insert("introspector_decl2_macros".to_string(), toml::Value::Table(macro_dep));

    // Dynamically add/update dependencies from patch_config.generated_crate_dependency
    for dep_entry in &patch_config.generated_crate_dependency {
        // Only process dependencies relevant to this specific crate
        if dep_entry.crate_name != paths.crate_name {
            continue;
        }

        let target_table = match dep_entry.section.as_str() {
            "dependencies" => &mut cargo_toml.dependencies,
            "dev-dependencies" => &mut cargo_toml.dev_dependencies,
            "build-dependencies" => &mut cargo_toml.build_dependencies,
            _ => {
                eprintln!("Warning: Unknown dependency section '{}' for crate '{}'", dep_entry.section, dep_entry.name);
                continue;
            }
        };

        let mut dep_table_value = toml::Table::new();
        if dep_entry.workspace {
            dep_table_value.insert("workspace".to_string(), toml::Value::Boolean(true));
        } else if let Some(version) = &dep_entry.version {
            dep_table_value.insert("version".to_string(), toml::Value::String(version.clone()));
        }

        if let Some(features) = &dep_entry.features {
            let features_array = toml::Value::Array(
                features.iter().map(|f| toml::Value::String(f.clone())).
                collect()
            );
            dep_table_value.insert("features".to_string(), features_array);
        }

        if let Some(package) = &dep_entry.package {
            dep_table_value.insert("package".to_string(), toml::Value::String(package.clone()));
        }

        target_table.insert(dep_entry.name.clone(), toml::Value::Table(dep_table_value));
    }


    // Remove [patch.crates-io] section from individual crate Cargo.toml
    cargo_toml.patch.clear();


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