use std::fs;
use anyhow::{Context, Result};
use crate::CratePaths;

/// Backs up original Cargo.toml file.
pub fn backup_original_cargotoml(paths: &CratePaths, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Dry-run: Would have backed up {} to {}", paths.cargo_toml_path.display(), paths.old_cargo_toml_path.display());
        return Ok(());
    }
    // Only perform the backup if the target Cargo.toml exists and a backup doesn't already exist.
    if paths.cargo_toml_path.exists() && !paths.old_cargo_toml_path.exists() {
        fs::rename(&paths.cargo_toml_path, &paths.old_cargo_toml_path)
            .context(format!("Failed to rename {} to {}", paths.cargo_toml_path.display(), paths.old_cargo_toml_path.display()))?;
        println!("Renamed {} to {}", paths.cargo_toml_path.display(), paths.old_cargo_toml_path.display());
    } else if paths.old_cargo_toml_path.exists() {
        println!("Backup {} already exists, skipping rename of {}", paths.old_cargo_toml_path.display(), paths.cargo_toml_path.display());
    } else if !paths.cargo_toml_path.exists() {
        // If no Cargo.toml exists and no backup exists, create an empty oldCargo.toml
        fs::write(&paths.old_cargo_toml_path, "")?;
        println!("No {} found, created empty {}", paths.cargo_toml_path.display(), paths.old_cargo_toml_path.display());
    }
    Ok(())
}
