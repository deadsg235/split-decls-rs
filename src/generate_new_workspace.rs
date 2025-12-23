use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Generates a Cargo.toml for a new workspace root.
/// Takes the output directory and a list of paths to the member crates.
pub fn generate_root_toml(
    output_dir: &Path,
    crate_paths: & [PathBuf],
    dry_run: bool,
) -> Result<()> {
    let mut workspace_members_content = Vec::new();
    for crate_path in crate_paths {
        // Assume crate_path is relative to the workspace root already, or resolve it
        let relative_path = crate_path.strip_prefix(output_dir)
            .unwrap_or(crate_path) // Fallback if not a prefix
            .display()
            .to_string();
        workspace_members_content.push(format!("\"{}\"", relative_path));
    }

    let workspace_cargo_toml_content = format!(
        r#"[workspace]\nresolver = \"2\"\nmembers = [\n    {}\n]\n"#,
        workspace_members_content.join(",\n    "),
    );

    let workspace_cargo_toml_path = output_dir.join("Cargo.toml");
    if dry_run {
        println!(
            "Dry-run: Would write workspace Cargo.toml to {}",
            workspace_cargo_toml_path.display()
        );
        println!("{}", workspace_cargo_toml_content);
    } else {
        fs::write(&workspace_cargo_toml_path, workspace_cargo_toml_content)
            .context(format!(
                "Failed to write Cargo.toml for new workspace: {}",
                workspace_cargo_toml_path.display()
            ))?;
        println!(
            "Generated workspace Cargo.toml at: {}",
            workspace_cargo_toml_path.display()
        );
    }

    Ok(())
}
