use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::patch_config;
use split_decls_types::SplitDeclsConfig;

use crate::generate_wrapped_crate::generate_wrapped_crate;

/// Generates a new workspace containing "wrapped" versions of the target crates.
/// Each wrapped crate will have its declarations eagerly split and patched.
pub fn generate_wrapped_workspace(
    output_dir: &Path,
    patch_config: &patch_config::PatchConfig,
    global_config: &SplitDeclsConfig,
    current_crate_name: &str, // Name of the split-decls-rs tool crate
    dry_run: bool,
) -> Result<()> {
    if !dry_run {
        fs::create_dir_all(output_dir)
            .context(format!("Failed to create wrapped workspace directory: {}", output_dir.display()))?;
    }
    println!("Wrapped workspace directory: {}", output_dir.display());

    let mut workspace_members = Vec::new();
    for target in &patch_config.targets {
        let wrapped_crate_name = format!("wrapped-{}", target.name);
        workspace_members.push(format!("\"{}\"", wrapped_crate_name));
        
        // Call a helper function to generate the individual wrapped crate
        generate_wrapped_crate(
            output_dir,
            target,
            global_config,
            current_crate_name,
            &wrapped_crate_name,
            dry_run,
        )?;
    }

    // Generate root Cargo.toml for the wrapped workspace
    let mut workspace_cargo_toml_content = format!(
        r#"[workspace]
members = [
    {}
]
"#,
        workspace_members.join(",\n    ")
    );

    // Always add [workspace.dependencies] section
    workspace_cargo_toml_content.push_str("\n[workspace.dependencies]\n");
    // Explicitly define all common dependencies with their versions or paths.
    // These values are based on typical versions found in the cargo2nix project submodules.
    // This is a fallback to ensure buildability when parent Cargo.toml is not a standard workspace.
    workspace_cargo_toml_content.push_str("proc-macro2 = { version = \"1.0\" }\n");
    workspace_cargo_toml_content.push_str("quote = { version = \"1.0\" }\n");
    workspace_cargo_toml_content.push_str("syn = { version = \"1.0\", features = [\"full\", \"visit\", \"visit-mut\"] }\n");
    workspace_cargo_toml_content.push_str("anyhow = { version = \"1.0\" }\n");
    workspace_cargo_toml_content.push_str("serde = { version = \"1.0\" }\n");
    workspace_cargo_toml_content.push_str("serde_derive = { version = \"1.0\" }\n");
    workspace_cargo_toml_content.push_str("toml = { version = \"0.5\" }\n");
    workspace_cargo_toml_content.push_str("walkdir = { version = \"2.3\" }\n");
    workspace_cargo_toml_content.push_str("tempfile = { version = \"3.2\" }\n");
    workspace_cargo_toml_content.push_str("url = { version = \"2.2\" }\n");
    workspace_cargo_toml_content.push_str("split-decls-types = { path = \"../split-decls-types\" }\n");


    // Generate [patch.crates-io] section based on patched_dependencies from PatchConfig
    if !patch_config.patched_dependencies.is_empty() {
        workspace_cargo_toml_content.push_str("\n[patch.crates-io]\n");
        for dep in &patch_config.patched_dependencies {
            // Only add introspector_decl* to patch, as others are now in workspace.dependencies
            if dep.name.starts_with("introspector_decl") || dep.name == "proc-macro2" {
                let mut dep_string = format!("{} = {{", dep.name);
                if let Some(version) = &dep.version {
                    dep_string.push_str(&format!(" version = \"{}\"", version));
                }
                if let Some(path) = &dep.path {
                    // The path in patch.toml is relative to the tool's root.
                    // The generated output/Cargo.toml is in `output/Cargo.toml`.
                    // So, the path needs to be adjusted to be relative to `output/Cargo.toml`.
                    let adjusted_path = format!("../{}", path.display()); // Fix relative path
                    dep_string.push_str(&format!(" path = \"{}\"", adjusted_path));
                }
                if let Some(features) = &dep.features {
                    dep_string.push_str(&format!(", features = [\"{}\"]", features.join("\", \"")));
                }
                dep_string.push_str(" }\n");
                workspace_cargo_toml_content.push_str(&dep_string);
            }
        }
    }




    let workspace_cargo_toml_path = output_dir.join("Cargo.toml");
    if !dry_run {
        fs::write(&workspace_cargo_toml_path, workspace_cargo_toml_content)
            .context(format!("Failed to write Cargo.toml for wrapped workspace: {}", workspace_cargo_toml_path.display()))?;
    }
    println!("Generated workspace Cargo.toml at: {}", workspace_cargo_toml_path.display());

    Ok(())
}
