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
    for (dep_name, dep_value) in &global_config.workspace_dependencies {
        workspace_cargo_toml_content.push_str(&format!("{} = {}\n", dep_name, dep_value.to_string()));
    }

    // Hardcoded fundamental dependencies that are often part of a workspace
    // or expected by proc-macros, to ensure the generated workspace builds.
    // These values are based on typical versions found in the cargo2nix project submodules.
    // This is a fallback to ensure buildability when parent Cargo.toml is not a standard workspace.
    if !global_config.workspace_dependencies.contains_key("quote") {
        workspace_cargo_toml_content.push_str("quote = { version = \"1.0\" }\n");
    }
    if !global_config.workspace_dependencies.contains_key("syn") {
        workspace_cargo_toml_content.push_str("syn = { version = \"1.0\", features = [\"full\", \"visit\", \"visit-mut\"] }\n");
    }
    if !global_config.workspace_dependencies.contains_key("anyhow") {
        workspace_cargo_toml_content.push_str("anyhow = { version = \"1.0\" }\n");
    }
    if !global_config.workspace_dependencies.contains_key("serde") {
        workspace_cargo_toml_content.push_str("serde = { version = \"1.0\" }\n");
    }
    if !global_config.workspace_dependencies.contains_key("serde_derive") {
        workspace_cargo_toml_content.push_str("serde_derive = { version = \"1.0\" }\n");
    }
    if !global_config.workspace_dependencies.contains_key("toml") {
        workspace_cargo_toml_content.push_str("toml = { version = \"0.5\" }\n");
    }
    if !global_config.workspace_dependencies.contains_key("walkdir") {
        workspace_cargo_toml_content.push_str("walkdir = { version = \"2.3\" }\n");
    }
    if !global_config.workspace_dependencies.contains_key("tempfile") {
        workspace_cargo_toml_content.push_str("tempfile = { version = \"3.2\" }\n");
    }
    if !global_config.workspace_dependencies.contains_key("url") {
        workspace_cargo_toml_content.push_str("url = { version = \"2.2\" }\n");
    }
    if !global_config.workspace_dependencies.contains_key("split-decls-types") {
        workspace_cargo_toml_content.push_str("split-decls-types = { path = \"../split-decls-types\" }\n");
    }

    // Explicitly add proc-macro2 and introspector_decl* as patches to workaround workspace dependency resolution issues
    // Paths are relative to the generated output directory.
    workspace_cargo_toml_content.push_str("\n[patch.crates-io]\n");
    if !global_config.workspace_dependencies.contains_key("proc-macro2") {
        workspace_cargo_toml_content.push_str("proc-macro2 = { path = \"../../submodules/proc-macro2\" }\n");
    }
    if !global_config.workspace_dependencies.contains_key("introspector_decl2_macros") {
        workspace_cargo_toml_content.push_str(&format!(
            "introspector_decl2_macros = {{ path = \"../../submodules/patch-build-rs/introspector_decl2_macros\" }}\n"
        ));
    }
    if !global_config.workspace_dependencies.contains_key("introspector_decl_common") {
        workspace_cargo_toml_content.push_str(&format!(
            "introspector_decl_common = {{ path = \"../../submodules/patch-build-rs/introspector_decl_common\" }}\n"
        ));
    }
    if !global_config.workspace_dependencies.contains_key("introspector_decl_core") {
        workspace_cargo_toml_content.push_str(&format!(
            "introspector_decl_core = {{ path = \"../../submodules/patch-build-rs/introspector_decl_core\" }}\n"
        ));
    }
    if !global_config.workspace_dependencies.contains_key("introspector_macro_helpers") {
        workspace_cargo_toml_content.push_str(&format!(
            "introspector_macro_helpers = {{ path = \"../../submodules/patch-build-rs/introspector_macro_helpers\" }}\n"
        ));
    }

    let workspace_cargo_toml_path = output_dir.join("Cargo.toml");
    if !dry_run {
        fs::write(&workspace_cargo_toml_path, workspace_cargo_toml_content)
            .context(format!("Failed to write Cargo.toml for wrapped workspace: {}", workspace_cargo_toml_path.display()))?;
    }
    println!("Generated workspace Cargo.toml at: {}", workspace_cargo_toml_path.display());

    Ok(())
}
