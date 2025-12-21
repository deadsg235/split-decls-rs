use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;
use split_decls_types::SplitDeclsConfig;

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
    let temp_path = temp_dir.path(); // This is the temporary workspace root
    println!("Test workspace created at: {}", temp_path.display());

    // Define paths within the temporary workspace
    let test_crate_name = "unimacro_derive";
    let original_test_crate_path = PathBuf::from("../../unimacro_derive");
    let original_patch_build_rs_path = PathBuf::from("../patch-build-rs"); // Path to the patch-build-rs directory
    let temp_test_crate_path = temp_path.join(test_crate_name); // This is where unimacro_derive will be copied
    let temp_patch_build_rs_path = temp_path.join("patch-build-rs"); // This is where patch-build-rs will be copied
    let temp_global_config_path = temp_path.join("split-decls-rs.toml");
    let temp_workspace_cargo_toml = temp_path.join("Cargo.toml"); // Workspace root Cargo.toml

    // Create the workspace root Cargo.toml
    let workspace_cargo_toml_content = format!(r#"
        [workspace]
        members = [
            "{}",
            "patch-build-rs/introspector_decl2_macros",
            "patch-build-rs/introspector_decl_core",
            "patch-build-rs/introspector_macro_helpers",
            "patch-build-rs/introspector_decl_common",
        ]

        [workspace.dependencies]
        proc-macro2 = {{ version = "1.0" }}
        quote = {{ version = "1.0" }}
        syn = {{ version = "2.0", features = ["full", "extra-traits", "visit", "fold", "visit-mut"] }}
        anyhow = {{ version = "1.0" }}
        toml = {{ version = "0.8" }}
        serde = {{ version = "1.0", features = ["derive"] }}

    fs::write(&temp_workspace_cargo_toml, workspace_cargo_toml_content)
        .context("Failed to write temporary workspace Cargo.toml")?;
    println!("Temporary workspace Cargo.toml created at: {}", temp_workspace_cargo_toml.display());

    // Copy the patch-build-rs directory into the temporary workspace
    println!("Copying patch-build-rs from {} to {}", original_patch_build_rs_path.display(), temp_patch_build_rs_path.display());
    copy_dir_recursive(&original_patch_build_rs_path, &temp_patch_build_rs_path)
        .context(format!("Failed to copy patch-build-rs from {} to {}", original_patch_build_rs_path.display(), temp_patch_build_rs_path.display()))?;

    // Copy the unimacro_derive crate into the temporary workspace
    println!("Copying test crate from {} to {}", original_test_crate_path.display(), temp_test_crate_path.display());
    copy_dir_recursive(&original_test_crate_path, &temp_test_crate_path)
        .context(format!("Failed to copy test crate from {} to {}", original_test_crate_path.display(), temp_test_crate_path.display()))?;

    // Create a dummy split-decls-rs.toml
    let dummy_config_content = format!(r#"
        active_overlay_modules = []
        custom_prelude_overlay = "introspector_decl2_macros::prelude"
        crates_io_patches = {{}} # Added missing field
        [[patches."{}"]] # Corrected to array of tables
        path = "dummy_patch.rs" # Placeholder
    "#, test_crate_name);
    fs::write(&temp_global_config_path, dummy_config_content)
        .context("Failed to write dummy split-decls-rs.toml")?;
    println!("Dummy split-decls-rs.toml created at: {}", temp_global_config_path.display());

    // Phase 2: Run split-decls-rs on the temporary workspace
    // This means calling the process_crates_in_path function
    println!("Running split-decls-rs on the temporary workspace...");
    
    // Load the dummy config
    let global_config: SplitDeclsConfig = toml::from_str(&fs::read_to_string(&temp_global_config_path)?)?;

    process_crates_in_path(
        &temp_path,         // Root path for processing crates (this is now the workspace root)
        "split-decls-rs",   // Current crate name (to skip itself)
        &global_config,     // Global config
        false,              // Not rustc source
        false,              // dry_run = false for actual execution
    ).context("split-decls-rs execution failed")?;
    println!("split-decls-rs completed successfully on the test crate.");

    // Phase 3: Compile the modified unimacro_derive from the workspace root
    println!("Attempting to build the modified test crate from workspace root: {}", temp_path.display());
    let output = Command::new("cargo")
        .arg("check") // Use check for faster validation
        .arg("-p") // Specify package within workspace
        .arg(test_crate_name)
        .current_dir(&temp_path) // Run from the workspace root
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
