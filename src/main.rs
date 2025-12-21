use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use split_decls_rs::git_manager;
use split_decls_rs::patch_config::{PatchConfig, PatchTarget};
use split_decls_types::SplitDeclsConfig;
use split_decls_rs::workspace_manager;
use walkdir::WalkDir; // Added for parsing crates in path in dry-run
use toml; // Import toml crate for parsing
use std::collections::HashMap; // Added
use std::fs; // Added

// Helper struct for parsing relevant parts of the root Cargo.toml
#[derive(Debug, serde::Deserialize)]
struct RootCargoToml {
    workspace: Option<RootWorkspace>,
}

#[derive(Debug, serde::Deserialize)]
struct RootWorkspace {
    #[serde(default)]
    dependencies: HashMap<String, toml::Value>,
}

fn main() -> Result<()> {
    println!("Starting split-decls-rs tool...");

    let mut args: Vec<String> = std::env::args().collect();
    println!("Parsed args: {:?}", args);

    let dry_run_index = args.iter().position(|arg| arg == "--dry-run");
    let dry_run = dry_run_index.is_some();

    // Remove --dry-run from args if present, so it doesn't interfere with path parsing
    if let Some(index) = dry_run_index {
        args.remove(index);
    }
    
    let verbose_index = args.iter().position(|arg| arg == "--verbose");
    let verbose = verbose_index.is_some();

    if let Some(index) = verbose_index {
        args.remove(index);
    }
    println!("dry_run flag: {}", dry_run);
    println!("verbose flag: {}", verbose);
    
    // Now get the patch config path
    // args[0] is typically the executable name "split-decls-rs"
    // args[1] would be the first actual argument after the executable
    let patch_config_path_str = args.get(1).map_or("patch.toml", |s| s.as_str());

    if dry_run {
        println!("*** Running in DRY-RUN mode. No files will be modified. ***");
    }

    let workspace_root = PathBuf::from("../../"); // Relative to current crate (split-decls-rs)
    let global_config_path = workspace_root.join("split-decls-rs.toml");
    let root_cargo_toml_path = workspace_root.join("Cargo.toml"); // Uncommented this line
    let mut global_config = SplitDeclsConfig::load_from_file(&global_config_path)
        .context("Failed to load global split-decls-rs config")?;
    println!("Global config loaded: {:?}", global_config);

    // --- Load root Cargo.toml and extract workspace dependencies ---
    let root_cargo_toml_content = fs::read_to_string(&root_cargo_toml_path)
        .context(format!("Failed to read root Cargo.toml from {}", root_cargo_toml_path.display()))?;
    
    let root_cargo_toml: RootCargoToml = toml::from_str(&root_cargo_toml_content)
        .context(format!("Failed to parse root Cargo.toml from {}", root_cargo_toml_path.display()))?;

    if let Some(root_workspace) = root_cargo_toml.workspace {
        global_config.workspace_dependencies = root_workspace.dependencies;
    }
    // --- END new logic ---

    // --- Manage workspace dependencies in the root Cargo.toml ---
    let root_cargo_toml_path = workspace_root.join("Cargo.toml");
    // let workspace_managed_deps: Vec<(String, toml::Value)> = vec![
    //     ("anyhow".to_string(), toml::Value::Table(toml::Table::from_iter([
    //         ("path".to_string(), toml::Value::String("submodules/anyhow".to_string())),
    //     ]))),
    //     ("proc-macro2".to_string(), toml::Value::Table(toml::Table::from_iter([
    //         ("path".to_string(), toml::Value::String("submodules/proc-macro2".to_string())),
    //     ]))),
    //     ("quote".to_string(), toml::Value::Table(toml::Table::from_iter([
    //         ("path".to_string(), toml::Value::String("submodules/quote".to_string())),
    //     ]))),
    //     ("syn".to_string(), toml::Value::Table(toml::Table::from_iter([
    //         ("path".to_string(), toml::Value::String("submodules/syn".to_string())),
    //     ]))),
    //     ("url".to_string(), toml::Value::Table(toml::Table::from_iter([
    //         ("version".to_string(), toml::Value::String("2.0".to_string())),
    //     ]))),
    //     ("walkdir".to_string(), toml::Value::Table(toml::Table::from_iter([
    //         ("version".to_string(), toml::Value::String("2.5".to_string())),
    //     ]))),
    //     ("toml".to_string(), toml::Value::Table(toml::Table::from_iter([
    //         ("version".to_string(), toml::Value::String("0.8".to_string())),
    //     ]))),
    //     ("serde".to_string(), toml::Value::Table(toml::Table::from_iter([
    //         ("version".to_string(), toml::Value::String("1.0".to_string())),
    //         ("features".to_string(), toml::Value::Array(vec![
    //             toml::Value::String("derive".to_string()),
    //         ])),
    //     ]))),
    //     ("serde_json".to_string(), toml::Value::Table(toml::Table::from_iter([
    //         ("version".to_string(), toml::Value::String("1.0".to_string())),
    //     ]))),
    //     ("once_cell".to_string(), toml::Value::Table(toml::Table::from_iter([
    //         ("version".to_string(), toml::Value::String("1.19".to_string())),
    //     ]))),
    //     ("introspector_decl2_macros".to_string(), toml::Value::Table(toml::Table::from_iter([
    //         ("path".to_string(), toml::Value::String("submodules/patch-build-rs/introspector_decl2_macros".to_string())),
    //     ]))),
    //     ("introspector_decl_core".to_string(), toml::Value::Table(toml::Table::from_iter([
    //         ("path".to_string(), toml::Value::String("submodules/patch-build-rs/introspector_decl_core".to_string())),
    //     ]))),
    //     ("introspector_macro_helpers".to_string(), toml::Value::Table(toml::Table::from_iter([
    //         ("path".to_string(), toml::Value::String("submodules/patch-build-rs/introspector_macro_helpers".to_string())),
    //     ]))),
    //     ("introspector_decl_common".to_string(), toml::Value::Table(toml::Table::from_iter([
    //         ("path".to_string(), toml::Value::String("submodules/patch-build-rs/introspector_decl_common".to_string())),
    //     ]))),
    // ];
    // workspace_manager::manage_workspace_dependencies(&root_cargo_toml_path, &workspace_managed_deps, dry_run)
    //     .context("Failed to manage workspace dependencies in root Cargo.toml")?;

    // --- Apply workspace package defaults to the root Cargo.toml ---
    // workspace_manager::apply_workspace_package_defaults_to_root(&root_cargo_toml_path, dry_run, verbose)
    //     .context("Failed to apply workspace package defaults to root Cargo.toml")?;

    // --- END NEW ---


let patch_config_path = PathBuf::from(patch_config_path_str);
    let patch_config = PatchConfig::load_from_file(&patch_config_path)
        .context(format!("Failed to load patch config from {}", patch_config_path.display()))?;
    println!("Patch config loaded: {:?}", patch_config);

    let current_crate_name = "split-decls-rs"; // This tool's crate name

    for target in patch_config.targets {
        println!("\n=== Processing target: {} ===", target.name);

        let target_path = workspace_root.join(&target.path);

        if let Some(repo_url) = target.repo_url {
            let git_reference = target.git_reference.unwrap_or_else(|| "main".to_string());
            println!("Managing Git repo for {} from {} at reference {}", target.name, repo_url, git_reference);

            if !dry_run {
                git_manager::manage_git_repo(
                    &repo_url,
                    &target_path,
                    &git_reference,
                    global_config.github_org.as_deref(),
                    &global_config.repo_fork_mapping,
                )?;
            } else {
                println!("Dry-run: Skipped Git repo management for {}", target.name);
            }

            // After managing the Git repo, process crates within its path
            // For external repos, we assume the root of the cloned repo might contain multiple crates
            split_decls_rs::process_crates_in_path(&target_path, current_crate_name, &global_config, false, dry_run)?;
        } else {
            // It's a local path, process it directly
            println!("Processing local crate: {}", target.name);
            split_decls_rs::process_crate(&target_path, &global_config, dry_run)?;
        }
        
        // --- Build the processed crate to verify generation ---
        println!("\n=== Building processed crate: {} ===", target.name);
        if !dry_run {
            let status = Command::new("cargo")
                .arg("build")
                .arg("--manifest-path")
                .arg(target_path.join("Cargo.toml"))
                .status()
                .context(format!("Failed to build crate {}", target.name))?;

            if !status.success() {
                anyhow::bail!("Building crate {} failed with status: {:?}", target.name, status);
            }
            println!("Successfully built processed crate: {}", target.name);
        } else {
            println!("Dry-run: Skipped building processed crate {}.", target.name);
        }
    }

    println!("\nSplit-decls-rs tool finished.");
    Ok(())
}
