use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

// Import the main function from our crate
use split_decls_rs::process_crates_in_path; // Assuming process_crates_in_path is public

// Test for the overall functionality:
// 1. Create a temporary workspace.
// 2. Copy a test crate (e.g., unimacro_derive) into it.
// 3. Create a dummy split-decls-rs.toml.
// 4. Run split-decls-rs on the temporary workspace.
// 5. Attempt to build the modified test crate.
#[test]
fn integration_test_unimacro_derive_build() -> Result<()> {
    // Phase 1: Setup a temporary workspace
    let temp_dir = tempdir().context("Failed to create temporary directory")?;
    let temp_path = temp_dir.path();
    println!("Test workspace created at: {}", temp_path.display());

    // Define paths within the temporary workspace
    let test_crate_name = "unimacro_derive";
    let original_test_crate_path = PathBuf::from("../../unimacro_derive");
    let temp_test_crate_path = temp_path.join(test_crate_name);
    let temp_global_config_path = temp_path.join("split-decls-rs.toml");

    // Copy the unimacro_derive crate into the temporary workspace
    // This requires recursive copy
    println!("Copying test crate from {} to {}", original_test_crate_path.display(), temp_test_crate_path.display());
    copy_dir_recursive(&original_test_crate_path, &temp_test_crate_path)
        .context(format!("Failed to copy test crate from {} to {}", original_test_crate_path.display(), temp_test_crate_path.display()))?;

    // Create a dummy split-decls-rs.toml
    let dummy_config_content = format!(r#"
        active_overlay_modules = []
        custom_prelude_overlay = "introspector_decl2_macros::prelude"
        [patches."{}"]
        # Add a dummy patch for unimacro_derive to satisfy config, if needed
        # path = "some/dummy_patch.rs"
    "#, test_crate_name);
    fs::write(&temp_global_config_path, dummy_config_content)
        .context("Failed to write dummy split-decls-rs.toml")?;
    println!("Dummy split-decls-rs.toml created at: {}", temp_global_config_path.display());

    // Phase 2: Run split-decls-rs on the temporary workspace
    // This means calling the process_crates_in_path function
    println!("Running split-decls-rs on the temporary workspace...");
    
    // Load the dummy config
    let global_config: split_decls_rs::config::SplitDeclsConfig = toml::from_str(&fs::read_to_string(&temp_global_config_path)?)?;

    process_crates_in_path(
        &temp_path,         // Root path for processing crates
        "split-decls-rs",   // Current crate name (to skip itself)
        &global_config,     // Global config
        false,              // Not rustc source
    ).context("split-decls-rs execution failed")?;
    println!("split-decls-rs completed successfully on the test crate.");

    // Phase 3: Compile the modified unimacro_derive
    println!("Attempting to build the modified test crate: {}", temp_test_crate_path.display());
    let output = Command::new("cargo")
        .arg("check") // Use check for faster validation
        .current_dir(&temp_test_crate_path)
        .output()
        .context("Failed to execute cargo check on modified crate")?;

    if !output.status.success() {
        eprintln!("Cargo check failed for {}:", temp_test_crate_path.display());
        eprintln!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        anyhow::bail!("Cargo check failed for modified test crate.");
    }

    println!("Modified test crate compiled successfully: {}", temp_test_crate_path.display());

    // Phase 4: Cleanup (temp_dir is automatically cleaned up when it goes out of scope)
    Ok(())
}

// Helper function for recursive directory copy
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst).context(format!("Failed to create destination directory {}", dst.display()))?;
    for entry in fs::read_dir(src).context(format!("Failed to read source directory {}", src.display()))? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(&entry.path(), &dst.join(entry.file_name()))
                .context(format!("Failed to copy file from {} to {}", entry.path().display(), dst.join(entry.file_name()).display()))?;
        }
    }
    Ok(())
}
