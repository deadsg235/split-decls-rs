use anyhow::{Context, Result};
use std::path::PathBuf;
use split_decls_rs::patch_config::PatchConfig;
use split_decls_types::SplitDeclsConfig;
use split_decls_rs::generate_wrapped_workspace::generate_wrapped_workspace;
use split_decls_rs::buildrs_generator::build_script_composer;
use split_decls_rs::{setup_crate_paths, eager_splitter};
use toml;
use std::fs;
use cargo_toml_generator_types::{CargoToml, Dependency};
use walkdir;

/// Helper function to convert an iterator of (String, cargo_toml_generator_types::Dependency)
/// to an iterator of (String, toml::Value).
fn dep_to_toml_value_iter<'a>(
    iter: impl IntoIterator<Item = (String, Dependency)> + 'a,
) -> impl Iterator<Item = (String, toml::Value)> + 'a {
    iter.into_iter().filter_map(|(name, dep)| {
        match toml::to_string(&dep) {
            Ok(serialized_dep) => {
                match toml::from_str(&serialized_dep) {
                    Ok(toml_value) => Some((name, toml_value)),
                    Err(_) => {
                        eprintln!("Warning: Failed to parse serialized Dependency TOML for {}", name);
                        None
                    }
                }
            }
            Err(_) => {
                eprintln!("Warning: Failed to serialize Dependency to TOML for {}", name);
                None
            }
        }
    })
}

fn main() -> Result<()> {
    println!("Starting split-decls-rs tool...");
    
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    println!("Parsed args: {:?}", args);
    
    let mut dry_run = false;
    let mut verbose = false;
    let mut target_dir_override: Option<String> = None;
    let mut recursive_override: Option<bool> = None;
    
    // Simple argument parsing
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--dry-run" => dry_run = true,
            "--verbose" => verbose = true,
            "--target-dir" => {
                if i + 1 < args.len() {
                    target_dir_override = Some(args[i + 1].clone());
                    i += 1;
                }
            },
            "--recursive" => recursive_override = Some(true),
            _ => {}
        }
        i += 1;
    }
    
    println!("dry_run flag: {}", dry_run);
    println!("verbose flag: {}", verbose);

    let wrapped_workspace_output_dir = PathBuf::from("output2");
    let patch_config_path_str = "patch.toml";

    if dry_run {
        println!("*** Running in DRY-RUN mode. No files will be modified. ***");
    }

    let workspace_root = PathBuf::from("./");
    let global_config_path = workspace_root.join("split-decls-rs.toml");
    let root_cargo_toml_path = workspace_root.join("output2/Cargo.toml");
    
    let mut global_config = if global_config_path.exists() {
        SplitDeclsConfig::load_from_file(&global_config_path)
            .context("Failed to load global split-decls-rs config")?
    } else {
        println!("No split-decls-rs.toml found at {}, using default configuration.", global_config_path.display());
        SplitDeclsConfig::default()
    };
    
    println!("Global config loaded: {:?}", global_config);

    // --- Load generated Cargo.toml and extract workspace dependencies ---
    let root_cargo_toml_content = if root_cargo_toml_path.exists() {
        fs::read_to_string(&root_cargo_toml_path)
            .context(format!("Failed to read generated Cargo.toml from {}", root_cargo_toml_path.display()))?
    } else {
        println!("No output2/Cargo.toml found, using target directory Cargo.toml");
        let target_dir = target_dir_override.as_deref().unwrap_or("../../");
        let target_cargo_path = PathBuf::from(target_dir).join("Cargo.toml");
        fs::read_to_string(&target_cargo_path)
            .context(format!("Failed to read target Cargo.toml from {}", target_cargo_path.display()))?
    };

    // Use the new CargoToml struct
    let root_cargo_toml: CargoToml = toml::from_str(&root_cargo_toml_content)
        .context(format!("Failed to parse generated Cargo.toml from {}", root_cargo_toml_path.display()))?;

    // Populate workspace dependencies from the generated CargoToml
    if let Some(workspace_section) = root_cargo_toml.workspace {
        global_config.workspace_dependencies.extend(dep_to_toml_value_iter(workspace_section.workspace_dependencies));
    }
    // Also extend with top-level dependencies, if any, for compatibility
    global_config.workspace_dependencies.extend(dep_to_toml_value_iter(root_cargo_toml.dependencies));
    global_config.workspace_dependencies.extend(dep_to_toml_value_iter(root_cargo_toml.dev_dependencies));
    global_config.workspace_dependencies.extend(dep_to_toml_value_iter(root_cargo_toml.build_dependencies)); // Include build-dependencies as well

    // Add dependencies from [patch] sections to workspace_dependencies
    if let Some(patch_section) = root_cargo_toml.patch {
        global_config.workspace_dependencies.extend(dep_to_toml_value_iter(patch_section.crates_io));
    }
    // --- END new logic ---

    let patch_config_path = PathBuf::from(patch_config_path_str);
    let patch_config = if patch_config_path.exists() {
        println!("No patch.toml found, using default patch configuration.");
        PatchConfig::default()
    } else {
        println!("No patch.toml found, using default patch configuration.");
        PatchConfig::default()
    };
    println!("Patch config loaded: {:?}", patch_config);

    let current_crate_name = "split-decls-rs"; // This tool's crate name

    // Always generate a wrapped workspace in this mode (hardcoded for now)
    println!("Generating wrapped workspace in: {}", wrapped_workspace_output_dir.display());
    generate_wrapped_workspace(
        &wrapped_workspace_output_dir,
        &patch_config,
        &global_config,
        current_crate_name,
        dry_run,
    )?;

    // --- NEW: Generate a sample build.rs using the new composer ---
    let target_build_rs_parts_dir = PathBuf::from("buildrs_parts_for_target_crate");
    let generated_target_build_rs_path = wrapped_workspace_output_dir.join("generated_target_build.rs");
    
    // Ensure the output directory exists
    fs::create_dir_all(&wrapped_workspace_output_dir)
        .context(format!("Failed to create output directory for target build.rs: {}", wrapped_workspace_output_dir.display()))?;

    println!("Attempting to compose target build.rs from parts in: {}", target_build_rs_parts_dir.display());
    build_script_composer::compose_build_script_from_parts(
        &target_build_rs_parts_dir,
        &generated_target_build_rs_path,
    )?;
    // --- END NEW ---

    // Process each crate for declaration splitting
    println!("Processing crates for declaration splitting...");
    
    // Find all Cargo.toml files recursively
    let mut crate_count = 0;
    for entry in walkdir::WalkDir::new(&workspace_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() == "Cargo.toml")
    {
        let cargo_toml_path = entry.path();
        let crate_path = cargo_toml_path.parent().unwrap();
        let lib_rs = crate_path.join("src/lib.rs");
        
        if lib_rs.exists() {
            crate_count += 1;
            let crate_name = crate_path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("unknown_crate_{}", crate_count));
            println!("Processing crate {}: {}", crate_count, crate_name);
            let paths = setup_crate_paths(&crate_path)?;
            eager_splitter::eager_split_crate(&paths, &global_config)?;
        }
    }

    // Process submodules with Rust crates
    println!("Processing submodules for declaration splitting...");
    let submodules_dir = workspace_root.join("submodules");
    if submodules_dir.exists() {
        let mut processed_count = 0;
        for entry in fs::read_dir(&submodules_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let crate_path = entry.path();
                let lib_rs = crate_path.join("src/lib.rs");
                let cargo_toml = crate_path.join("Cargo.toml");
                
                if lib_rs.exists() && cargo_toml.exists() {
                    println!("Processing submodule: {}", crate_path.file_name().unwrap().to_string_lossy());
                    let paths = setup_crate_paths(&crate_path)?;
                    eager_splitter::eager_split_crate(&paths, &global_config)?;
                    processed_count += 1;
                    
                    // Limit to prevent overwhelming output
                    if processed_count >= 50 {
                        println!("Processed 50 submodules, stopping to prevent overflow...");
                        break;
                    }
                }
            }
        }
    }

    println!("\nWrapped workspace generation finished.");
    println!("\nSplit-decls-rs tool finished.");
    Ok(())
}
