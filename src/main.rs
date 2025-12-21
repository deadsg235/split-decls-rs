use anyhow::{Context, Result};
use std::path::PathBuf;
 // Keep for now in case other parts of lib.rs use it for git
 // Keep for now
use split_decls_rs::patch_config::PatchConfig;
use split_decls_types::SplitDeclsConfig;
use split_decls_rs::generate_wrapped_workspace::generate_wrapped_workspace;
 // Keep for now
 // Keep for now
use toml;
use std::collections::HashMap;
use std::fs;

// mod cli_args; // Commented out: No longer needed with hardcoded args
// use cli_args::CliArgs; // Commented out: No longer needed

// Helper struct for parsing relevant parts of the root Cargo.toml
#[derive(Debug, serde::Deserialize)]
struct RootCargoToml {
    workspace: Option<RootWorkspace>,
    #[serde(default)]
    dependencies: HashMap<String, toml::Value>,
    #[serde(rename = "dev-dependencies", default)]
    dev_dependencies: HashMap<String, toml::Value>,
    #[serde(default)]
    patch: Option<PatchSection>, // Changed to Option<PatchSection>
}

#[derive(Debug, serde::Deserialize)]
struct RootWorkspace {
    #[serde(default)]
    dependencies: HashMap<String, toml::Value>,
}

#[derive(Debug, serde::Deserialize)]
struct PatchSection {
    #[serde(rename = "crates-io", default)]
    crates_io: HashMap<String, toml::Value>,
    // Potentially other patch sources like git
}

#[derive(Debug, serde::Deserialize)]
struct GenericCargoToml {
    #[serde(default)]
    dependencies: HashMap<String, toml::Value>,
    #[serde(rename = "dev-dependencies", default)]
    dev_dependencies: HashMap<String, toml::Value>,
    // We don't care about build-dependencies at the root level for workspace inheritance for now
}

fn main() -> Result<()> {
    println!("Starting split-decls-rs tool...");

    // Commented out: Argument parsing is removed for minimal edits
    // let cli_args = cli_args::parse_args()?;
    // println!("Parsed args: dry_run={}, verbose={}, output_dir={}, patch_config={}, generate_wrapped_workspace_mode={}",
    //          cli_args.dry_run, cli_args.verbose, cli_args.wrapped_workspace_output_dir.display(), cli_args.patch_config_path_str, cli_args.generate_wrapped_workspace_mode);

    // Hardcode values for minimal execution
    let dry_run = false;
    let _verbose = false;
    let wrapped_workspace_output_dir = PathBuf::from("output");
    let patch_config_path_str = "patch.toml";
    // let generate_wrapped_workspace_mode = true; // This will be implicitly true as we call the function directly


    if dry_run {
        println!("*** Running in DRY-RUN mode. No files will be modified. ***");
    }

    let workspace_root = PathBuf::from("./"); // Relative to current crate (split-decls-rs) - now project root
    let global_config_path = workspace_root.join("split-decls-rs.toml");
    let root_cargo_toml_path = workspace_root.join("Cargo.toml");
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
    } else {
        // If no [workspace] section, try to read top-level dependencies directly
        // This is done by `RootCargoToml` itself now
    }
    // Always add top-level dependencies from root Cargo.toml to workspace_dependencies
    global_config.workspace_dependencies.extend(root_cargo_toml.dependencies);
    global_config.workspace_dependencies.extend(root_cargo_toml.dev_dependencies);

    // Add dependencies from [patch] sections to workspace_dependencies
    if let Some(patch_section) = root_cargo_toml.patch {
        global_config.workspace_dependencies.extend(patch_section.crates_io);
    }
    // --- END new logic ---

    let patch_config_path = PathBuf::from(patch_config_path_str);
    let patch_config = PatchConfig::load_from_file(&patch_config_path)
        .context(format!("Failed to load patch config from {}", patch_config_path.display()))?;
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
    println!("\nWrapped workspace generation finished.");

    // Commented out: The previous loop for in-place processing is not needed for the current goal.
    // else {
    //     // This is the restored in-place processing loop, which will be moved into a function
    //     // `split_decls_rs::process_targets_in_place` later.
    //     for target in patch_config.targets {
    //         println!("\n=== Processing target: {} ===", target.name);

    //         let target_path = workspace_root.join(&target.path);

    //         if let Some(repo_url) = target.repo_url {
    //             let git_reference = target.git_reference.unwrap_or_else(|| "main".to_string());
    //             println!("Managing Git repo for {} from {} at reference {}", target.name, repo_url, git_reference);

    //             if !dry_run {
    //                 git_manager::manage_git_repo(
    //                     &repo_url,
    //                     &target_path,
    //                     &git_reference,
    //                     global_config.github_org.as_deref(),
    //                     &global_config.repo_fork_mapping,
    //                 )?;
    //             } else {
    //                 println!("Dry-run: Skipped Git repo management for {}", target.name);
    //             }

    //             // After managing the Git repo, process crates within its path
    //             // For external repos, we assume the root of the cloned repo might contain multiple crates
    //             split_decls_rs::process_crates_in_path(&target_path, current_crate_name, &global_config, false, dry_run)?;
    //         } else {
    //             // It's a local path, process it directly
    //             println!("Processing local crate: {}", target.name);
    //             split_decls_rs::process_crate(&target_path, &global_config, dry_run)?;
    //         }
            
    //         // --- Build the processed crate to verify generation ---
    //         println!("\n=== Building processed crate: {} ===", target.name);
    //         if !dry_run {
    //             let status = Command::new("cargo")
    //                 .arg("build")
    //                 .arg("--manifest-path")
    //                 .arg(target_path.join("Cargo.toml"))
    //                 .status()
    //                 .context(format!("Failed to build crate {}", target.name))?;

    //             if !status.success() {
    //                 anyhow::bail!("Building crate {} failed with status: {:?}", target.name, status);
    //             }
    //             println!("Successfully built processed crate: {}", target.name);
    //         } else {
    //             println!("Dry-run: Skipped building processed crate {}.", target.name);
    //         }
    //     }
    // }

    println!("\nSplit-decls-rs tool finished.");
    Ok(())
}
