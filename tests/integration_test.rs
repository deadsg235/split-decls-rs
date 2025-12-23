use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;
use split_decls_types::SplitDeclsConfig;

// Import the main function from our crate
use split_decls_rs::{process_crates_in_path, process_crate, generate_wrapped_workspace};

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

    "#, test_crate_name); // Fixed to include introspector crates
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

#[test]
fn test_eager_splitting_and_patching() -> Result<()> {
    let temp_dir = tempdir().context("Failed to create temporary directory")?;
    let test_crate_path = temp_dir.path().join("my_test_crate");
    fs::create_dir(&test_crate_path)?;
    fs::create_dir(test_crate_path.join("src"))?;

    // 1. Create a dummy Cargo.toml
    let cargo_toml_content = r#"
        [package]
        name = "my-test-crate"
        version = "0.1.0"
        edition = "2021"

        [dependencies]
        anyhow = "1.0"
        proc-macro2 = "1.0"
        quote = "1.0"
        syn = { version = "2.0", features = ["full", "extra-traits", "visit", "fold", "visit-mut"] }
        introspector_decl2_macros = { path = "../patch-build-rs/introspector_decl2_macros" }
    "#;
    fs::write(test_crate_path.join("Cargo.toml"), cargo_toml_content)?;

    // 2. Create a dummy src/lib.rs
    let lib_rs_content = r#"
        pub fn my_function(a: i32, b: i32) -> i32 {
            a + b
        }

        pub struct MyStruct {
            pub field: bool,
        }

        impl MyStruct {
            pub fn new() -> Self {
                MyStruct { field: true }
            }
        }
    "#;
    fs::write(test_crate_path.join("src").join("lib.rs"), lib_rs_content)?;

    // 3. Create a dummy patch file
    let patch_dir = temp_dir.path().join("patches");
    fs::create_dir(&patch_dir)?;
    let my_patch_path = patch_dir.join("my_patch.rs");
    let patch_content = r#"
        pub fn my_function(a: i32, b: i32) -> i32 {
            // Patched function
            a * b
        }
    "#;
    fs::write(&my_patch_path, patch_content)?;

    // Create a dummy patch-build-rs dir and macros crate for the test
    let patch_build_rs_dir = temp_dir.path().join("patch-build-rs");
    fs::create_dir_all(patch_build_rs_dir.join("introspector_decl2_macros"))?;
    fs::write(patch_build_rs_dir.join("introspector_decl2_macros/Cargo.toml"), r#"[package] name = "introspector_decl2_macros" version = "0.1.0" edition = "2021" [dependencies] proc-macro2 = "1.0" quote = "1.0" syn = { version = "2.0", features = ["full"] }"#)?;
    fs::write(patch_build_rs_dir.join("introspector_decl2_macros/src/lib.rs"), r#"pub mod prelude { pub use quote::quote; pub use syn::{parse_macro_input, Attribute, LitStr, Token}; } #[proc_macro] pub fn prelude(_input: proc_macro::TokenStream) -> proc_macro::TokenStream { proc_macro::TokenStream::new() } #[proc_macro_attribute] pub fn decl_MyTestCrate_decls_my_function(_attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream { item } "#)?;


    // 4. Create a dummy split-decls-rs.toml
    let global_config_content = format!(
        r#"
        custom_prelude_overlay = "// My custom prelude for eager splitting"

        [patches."my-test-crate"]
        path = "{}"
        "#,
        my_patch_path.to_str().context("Path not UTF-8")?
    );
    let global_config_path = temp_dir.path().join("split-decls-rs.toml");
    fs::write(&global_config_path, global_config_content)?;

    let global_config: SplitDeclsConfig =
        toml::from_str(&fs::read_to_string(&global_config_path)?)?;

    // 5. Run process_crate
    process_crate(&test_crate_path, &global_config, false)?;

    // 6. Assertions
    let decls_output_dir = test_crate_path.join("src").join("decls");
    assert!(decls_output_dir.exists());

    let my_function_decl_path = decls_output_dir.join("my_test_crate_decls_my_function.rs");
    assert!(my_function_decl_path.exists());
    let my_function_content = fs::read_to_string(&my_function_decl_path)?;
    assert!(my_function_content.contains("// My custom prelude for eager splitting"));
    // Verify patch was applied
    assert!(my_function_content.contains("a * b"));

    let my_struct_decl_path = decls_output_dir.join("my_test_crate_decls_MyStruct.rs");
    assert!(my_struct_decl_path.exists());
    let my_struct_content = fs::read_to_string(&my_struct_decl_path)?;
    assert!(my_struct_content.contains("// My custom prelude for eager splitting"));
    assert!(my_struct_content.contains("pub struct MyStruct"));

    let my_struct_impl_decl_path = decls_output_dir.join("my_test_crate_decls_impl_for_MyStruct.rs");
    assert!(my_struct_impl_decl_path.exists());
    let my_struct_impl_content = fs::read_to_string(&my_struct_impl_decl_path)?;
    assert!(my_struct_impl_content.contains("// My custom prelude for eager splitting"));
    assert!(my_struct_impl_content.contains("impl MyStruct"));

    let decl_invocation_path = decls_output_dir.join("_decl_module_invocation.rs");
    assert!(decl_invocation_path.exists());
    let decl_invocation_content = fs::read_to_string(&decl_invocation_path)?;
    assert!(decl_invocation_content.contains("decl_module!(my_test_crate_decls_my_function, my_test_crate_decls_MyStruct, my_test_crate_decls_impl_for_MyStruct);"));

    let generated_build_rs_path = test_crate_path.join("build.rs");
    assert!(generated_build_rs_path.exists());
    let build_rs_content = fs::read_to_string(&generated_build_rs_path)?;
    assert!(build_rs_content.contains("cargo:rerun-if-changed=build.rs"));
    assert!(build_rs_content.contains("cargo:rerun-if-changed=.split-decls-config.toml"));
    assert!(build_rs_content.contains(&format!("cargo:rerun-if-changed={}", my_patch_path.display())));
    assert!(!build_rs_content.contains("fs::create_dir_all")); // No longer creates dirs
    assert!(!build_rs_content.contains("lib.rs content")); // No longer reads lib.rs

    // Attempt to cargo check the generated crate
    let output = Command::new("cargo")
        .arg("check")
        .current_dir(&test_crate_path)
        .output()
        .context("Failed to execute cargo check on eager-split crate")?;

    if !output.status.success() {
        eprintln!("Cargo check failed for eager-split crate:");
        eprintln!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        anyhow::bail!("Cargo check failed for eager-split crate.");
    }

    Ok(())
}

#[test]
fn test_generate_wrapped_workspace() -> Result<()> {
    let temp_dir = tempdir().context("Failed to create temporary directory")?;
    let original_project_root = temp_dir.path().join("original_project");
    fs::create_dir(&original_project_root)?;

    // Create original_crate_a
    let original_crate_a_path = original_project_root.join("crate_a");
    fs::create_dir_all(original_crate_a_path.join("src"))?;
    fs::write(original_crate_a_path.join("Cargo.toml"), r#"
        [package]
        name = "crate-a"
        version = "0.1.0"
        edition = "2021"
        [dependencies]
        anyhow = "1.0"
        crate-b = { path = "../crate_b" }
    "#)?;
    fs::write(original_crate_a_path.join("src/lib.rs"), r#"
        pub fn a_function() -> &'static str { "Hello from A" }
    "#)?;

    // Create original_crate_b
    let original_crate_b_path = original_project_root.join("crate_b");
    fs::create_dir_all(original_crate_b_path.join("src"))?;
    fs::write(original_crate_b_path.join("Cargo.toml"), r#"
        [package]
        name = "crate-b"
        version = "0.1.0"
        edition = "2021"
        [dependencies]
        anyhow = "1.0"
    "#)?;
    fs::write(original_crate_b_path.join("src/lib.rs"), r#"
        pub fn b_function() -> &'static str { "Hello from B" }
    "#)?;

    // Create a dummy patch-build-rs dir and macros crate for the test
    let patch_build_rs_dir = temp_dir.path().join("patch-build-rs");
    fs::create_dir_all(patch_build_rs_dir.join("introspector_decl2_macros"))?;
    fs::write(patch_build_rs_dir.join("introspector_decl2_macros/Cargo.toml"), r#"[package] name = "introspector_decl2_macros" version = "0.1.0" edition = "2021" [dependencies] proc-macro2 = "1.0" quote = "1.0" syn = { version = "2.0", features = ["full"] }"#)?;
    fs::write(patch_build_rs_dir.join("introspector_decl2_macros/src/lib.rs"), r#"pub mod prelude { pub use quote::quote; pub use syn::{parse_macro_input, Attribute, LitStr, Token}; } #[proc_macro] pub fn prelude(_input: proc_macro::TokenStream) -> proc_macro::TokenStream { proc_macro::TokenStream::new() } #[proc_macro_attribute] pub fn decl_CrateA_decls_a_function(_attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream { item } #[proc_macro_attribute] pub fn decl_CrateB_decls_b_function(_attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream { item }"#)?;


    // Create a dummy split-decls-rs.toml
    let global_config_content = r#"
        custom_prelude_overlay = "// Custom prelude for wrapped workspace"
    "#;
    let global_config_path = original_project_root.join("split-decls-rs.toml");
    fs::write(&global_config_path, global_config_content)?;
    let global_config: SplitDeclsConfig = toml::from_str(&fs::read_to_string(&global_config_path)?)?;

    // Create a patch.toml (configures which crates to wrap)
    let patch_config_content = format!(r#"
        [[targets]]
        name = "crate-a"
        path = "{}"

        [[targets]]
        name = "crate-b"
        path = "{}"
    "#,
        original_crate_a_path.strip_prefix(original_project_root.parent().unwrap())?.display(),
        original_crate_b_path.strip_prefix(original_project_root.parent().unwrap())?.display(),
    );
    let patch_config_path = original_project_root.join("patch.toml");
    fs::write(&patch_config_path, patch_config_content)?;
    let patch_config = split_decls_rs::patch_config::PatchConfig::load_from_file(&patch_config_path)?;


    let wrapped_workspace_output_dir = temp_dir.path().join("wrapped-output");
    let current_crate_name = "split-decls-rs"; // This tool's crate name

    // Call generate_wrapped_workspace
    generate_wrapped_workspace(
        &wrapped_workspace_output_dir,
        &patch_config,
        &global_config,
        current_crate_name,
        false,
    )?;

    // Assertions for wrapped-workspace
    assert!(wrapped_workspace_output_dir.exists());
    let wrapped_workspace_cargo_toml = wrapped_workspace_output_dir.join("Cargo.toml");
    assert!(wrapped_workspace_cargo_toml.exists());
    let ws_cargo_content = fs::read_to_string(&wrapped_workspace_cargo_toml)?;
    assert!(ws_cargo_content.contains(r#"members = ["wrapped-crate-a", "wrapped-crate-b"]"#));

    // Assertions for wrapped-crate-a
    let wrapped_crate_a_path = wrapped_workspace_output_dir.join("wrapped-crate-a");
    assert!(wrapped_crate_a_path.exists());
    let wrapped_crate_a_cargo_toml = wrapped_crate_a_path.join("Cargo.toml");
    assert!(wrapped_crate_a_cargo_toml.exists());
    let a_cargo_content = fs::read_to_string(&wrapped_crate_a_cargo_toml)?;
    assert!(a_cargo_content.contains(r#"name = "wrapped-crate-a""#));
    // Verify path dependency to crate-b is absolute
    assert!(a_cargo_content.contains(&format!(r#"crate-b = {{ path = "{}" }}"#, original_crate_b_path.canonicalize()?.display())));

    let wrapped_crate_a_decls_dir = wrapped_crate_a_path.join("src/decls");
    assert!(wrapped_crate_a_decls_dir.exists());
    assert!(wrapped_crate_a_decls_dir.join("wrapped_crate_a_decls_a_function.rs").exists());
    let a_func_content = fs::read_to_string(wrapped_crate_a_decls_dir.join("wrapped_crate_a_decls_a_function.rs"))?;
    assert!(a_func_content.contains("// Custom prelude for wrapped workspace"));

    // Assertions for wrapped-crate-b
    let wrapped_crate_b_path = wrapped_workspace_output_dir.join("wrapped-crate-b");
    assert!(wrapped_crate_b_path.exists());
    let wrapped_crate_b_cargo_toml = wrapped_crate_b_path.join("Cargo.toml");
    assert!(wrapped_crate_b_cargo_toml.exists());
    let b_cargo_content = fs::read_to_string(&wrapped_crate_b_cargo_toml)?;
    assert!(b_cargo_content.contains(r#"name = "wrapped-crate-b""#));

    let wrapped_crate_b_decls_dir = wrapped_crate_b_path.join("src/decls");
    assert!(wrapped_crate_b_decls_dir.exists());
    assert!(wrapped_crate_b_decls_dir.join("wrapped_crate_b_decls_b_function.rs").exists());
    let b_func_content = fs::read_to_string(wrapped_crate_b_decls_dir.join("wrapped_crate_b_decls_b_function.rs"))?;
    assert!(b_func_content.contains("// Custom prelude for wrapped workspace"));

    // Attempt to cargo check the entire wrapped workspace
    let output = Command::new("cargo")
        .arg("check")
        .current_dir(&wrapped_workspace_output_dir)
        .output()
        .context("Failed to execute cargo check on wrapped workspace")?;

    if !output.status.success() {
        eprintln!("Cargo check failed for wrapped workspace:");
        eprintln!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        anyhow::bail!("Cargo check failed for wrapped workspace.");
    }

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