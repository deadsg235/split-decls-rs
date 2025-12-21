/// Backs up original lib.rs and build.rs files.
pub fn backup_original_files(paths: &CratePaths, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Dry-run: Would have backed up {} to {} and {} to {}", paths.lib_rs_path.display(), paths.old_lib_rs_path.display(), paths.build_rs_path.display(), paths.old_build_rs_path.display());
        return Ok(());
    }
    // Backup lib.rs
    if paths.lib_rs_path.exists() && !paths.old_lib_rs_path.exists() {
        fs::rename(&paths.lib_rs_path, &paths.old_lib_rs_path)
            .context(format!("Failed to rename {} to {}", paths.lib_rs_path.display(), paths.old_lib_rs_path.display()))?;
        println!("Renamed {} to {}", paths.lib_rs_path.display(), paths.old_lib_rs_path.display());
    } else if paths.old_lib_rs_path.exists() {
        println!("Backup {} already exists, skipping rename of {}", paths.old_lib_rs_path.display(), paths.lib_rs_path.display());
    } else if !paths.lib_rs_path.exists() {
        // If neither lib.rs nor oldlib.rs exists, create an empty oldlib.rs
        fs::write(&paths.old_lib_rs_path, "")?;
        println!("No {} found, created empty {}", paths.lib_rs_path.display(), paths.old_lib_rs_path.display());
    }
    
    // Backup build.rs
    if paths.build_rs_path.exists() && !paths.old_build_rs_path.exists() {
        fs::rename(&paths.build_rs_path, &paths.old_build_rs_path)
            .context(format!("Failed to rename {} to {}", paths.build_rs_path.display(), paths.old_build_rs_path.display()))?;
        println!("Renamed {} to {}", paths.build_rs_path.display(), paths.old_build_rs_path.display());
    } else if paths.old_build_rs_path.exists() {
        println!("Backup {} already exists, skipping rename of {}", paths.old_build_rs_path.display(), paths.build_rs_path.display());
    } else if !paths.build_rs_path.exists() {
        // If neither build.rs nor oldbuild.rs exists, create an empty oldbuild.rs
        fs::write(&paths.old_build_rs_path, "")?;
        println!("No {} found, created empty {}", paths.build_rs_path.display(), paths.old_build_rs_path.display());
    }
    Ok(())
}
