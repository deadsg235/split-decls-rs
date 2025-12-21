use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;
use anyhow::Context;
use anyhow::Result;
/// Attempts to resolve the actual crate path within a submodule directory.
/// This handles cases where the [workspace.dependencies.<name>].path points to a submodule root,
/// but the actual crate (with its Cargo.toml) resides in a subdirectory.
/// It prioritizes subdirectories named after the crate, then performs a shallow search.
pub fn resolve_crate_path_in_submodule(submodule_path: &Path, crate_name: &str) -> Result<PathBuf> {
    // 1. Check if Cargo.toml exists directly at the submodule_path
    if submodule_path.join("Cargo.toml").exists() {
        return Ok(submodule_path.to_path_buf());
    }

    // 2. Check for a subdirectory with the same name as the crate
    let named_subdir_path = submodule_path.join(crate_name.replace('-', "_")); // Handle kebab-case
    if named_subdir_path.join("Cargo.toml").exists() {
        return Ok(named_subdir_path);
    }
    let named_subdir_path_kebab = submodule_path.join(crate_name); // Check original kebab-case too
    if named_subdir_path_kebab.join("Cargo.toml").exists() {
        return Ok(named_subdir_path_kebab);
    }

    // 3. Perform a shallow search for Cargo.toml in direct subdirectories
    for entry in WalkDir::new(submodule_path)
        .max_depth(2) // Search current directory and one level deep
        .into_iter()
        .filter_map(|e| e.ok()) {
        if entry.file_name() == "Cargo.toml" {
            let cargo_toml_path = entry.path();
            let parent_dir = cargo_toml_path.parent().context("Cargo.toml has no parent")?;
            // A heuristic: if the package name in that Cargo.toml matches the dep_name, use it.
            // This would require parsing the inner Cargo.toml, which is more complex.
            // For now, let's just return the first one found in a subdirectory.
            if parent_dir != submodule_path { // Ensure it's not the root itself (already checked)
                return Ok(parent_dir.to_path_buf());
            }
        }
    }

    // If no specific crate path is found, fall back to the provided submodule_path
    // This might still lead to an error later if cargo can't find the package,
    // but it's the best we can do without more specific config.
    Ok(submodule_path.to_path_buf())
}
