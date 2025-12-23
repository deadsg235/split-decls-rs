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
use clap::{Parser, Subcommand}; // Added clap imports
use std::time::Instant; // Added for ecosystem_scan_mode
use std::path::Path; // Added for ecosystem_scan_mode (Path type)
use split_decls_rs::goal_parser::{GoalConfig, Workflow};
use split_decls_rs::workflow_executor::WorkflowExecutor;
mod ecosystem_processor; // New module for ecosystem processing

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Turn on verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generates a wrapped workspace for given crates
    #[command(name = "wrapped-workspace")]
    WrappedWorkspace {
        /// Optional: Directory to output the wrapped workspace
        #[arg(short, long, value_name = "DIR")]
        output_dir: Option<PathBuf>,

        /// Run in dry-run mode, no files will be modified
        #[arg(short, long)]
        dry_run: bool,
    },
    /// Scans an ecosystem for crates and processes them
    #[command(name = "ecosystem-scan")]
    EcosystemScan {
        /// Base path to scan for crates
        #[arg(value_name = "PATH")]
        base_path: PathBuf,

        /// Run in dry-run mode, no files will be modified
        #[arg(short, long)]
        recursive: bool,

	/// Run in dry-run mode, no files will be modified
        #[arg(short, long)]
        dry_run: bool,

    },
    /// Executes a workflow defined in a goal.toml file
    #[command(name = "execute-goal-workflow")]
    ExecuteGoalWorkflow {
        /// Path to the goal.toml file
        #[arg(value_name = "FILE")]
        goal_file: PathBuf,
        /// Run in dry-run mode, no files will be modified
        #[arg(short, long)]
        dry_run: bool,
    },
    /// Bootstrap command: Scans the project itself into output2, builds the generated code, and reports errors.
    #[command(name = "bootstrap")]
    Bootstrap {
        /// Optional: Directory to output the wrapped workspace
        #[arg(short, long, value_name = "DIR")]
        output_dir: Option<PathBuf>,
        /// Run in dry-run mode, no files will be modified
        #[arg(short, long)]
        dry_run: bool,
    },
}

// Placeholder functions for modes
fn run_wrapped_workspace_mode(
    verbose: bool,
    dry_run: bool,
    output_dir_override: Option<&PathBuf>,
    global_config: &SplitDeclsConfig,
) -> Result<()> {
    if verbose {
        if dry_run {
            println!("*** Running in DRY-RUN mode. No files will be modified. ***");
        }
        println!("Running wrapped-workspace mode.");
    }

    let wrapped_workspace_output_dir = output_dir_override
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("output2"));
    
    let patch_config_path_str = "patch.toml";
    let workspace_root = PathBuf::from("./");

    let root_cargo_toml_path = wrapped_workspace_output_dir.join("Cargo.toml");
    
    let mut global_config_mut = global_config.clone(); // Clone to allow mutation

    // --- Load generated Cargo.toml and extract workspace dependencies ---
    let root_cargo_toml_content = if root_cargo_toml_path.exists() {
        fs::read_to_string(&root_cargo_toml_path)
            .context(format!("Failed to read generated Cargo.toml from {}", root_cargo_toml_path.display()))?
    } else {
        if verbose {
            println!("No {} found, using target directory Cargo.toml or default.", root_cargo_toml_path.display());
        }
        let target_dir = PathBuf::from("./"); // Default if not overridden
        let target_cargo_path = target_dir.join("Cargo.toml"); // This should probably be the root Cargo.toml of the project
        
        // This logic seems a bit off. It should likely load the *main* Cargo.toml of the project
        // if output2/Cargo.toml doesn't exist, to get workspace dependencies from it.
        // For now, mirroring original logic:
        if target_cargo_path.exists() {
            fs::read_to_string(&target_cargo_path)
                .context(format!("Failed to read target Cargo.toml from {}", target_cargo_path.display()))?
        } else {
            // Fallback: provide an empty Cargo.toml content to avoid crash if no Cargo.toml is found
            if verbose {
                println!("Warning: No root Cargo.toml found at {}, using empty content for dependency extraction.", target_cargo_path.display());
            }
            "[package]\nname = \"dummy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n".to_string()
        }
    };

    // Use the new CargoToml struct
    let root_cargo_toml: CargoToml = toml::from_str(&root_cargo_toml_content)
        .context(format!("Failed to parse generated Cargo.toml from {}", root_cargo_toml_path.display()))?
    ;

    // Populate workspace dependencies from the generated CargoToml
    if let Some(workspace_section) = root_cargo_toml.workspace {
        global_config_mut.workspace_dependencies.extend(dep_to_toml_value_iter(workspace_section.workspace_dependencies));
    }
    // Also extend with top-level dependencies, if any, for compatibility
    global_config_mut.workspace_dependencies.extend(dep_to_toml_value_iter(root_cargo_toml.dependencies));
    global_config_mut.workspace_dependencies.extend(dep_to_toml_value_iter(root_cargo_toml.dev_dependencies));
    global_config_mut.workspace_dependencies.extend(dep_to_toml_value_iter(root_cargo_toml.build_dependencies)); // Include build-dependencies as well

    // Add dependencies from [patch] sections to workspace_dependencies
    if let Some(patch_section) = root_cargo_toml.patch {
        global_config_mut.workspace_dependencies.extend(dep_to_toml_value_iter(patch_section.crates_io));
    }
    // --- END new logic ---

    let patch_config_path = PathBuf::from(patch_config_path_str);
    let patch_config = if patch_config_path.exists() {
        if verbose {
            println!("Loading patch config from {}", patch_config_path.display());
        }
        PatchConfig::load_from_file(&patch_config_path)
            .context("Failed to load patch config")?
    } else {
        if verbose {
            println!("No patch.toml found, using default patch configuration.");
        }
        PatchConfig::default()
    };
    if verbose {
        println!("Patch config loaded: {:?}", patch_config);
    }

    let current_dir_as_scan_root = PathBuf::from("./");

    // Always generate a wrapped workspace in this mode
    if verbose {
        println!("Generating wrapped workspace in: {}", wrapped_workspace_output_dir.display());
    }
    generate_wrapped_workspace(
        &wrapped_workspace_output_dir,
        &patch_config,
        &global_config_mut, // Use mutable clone here
        &current_dir_as_scan_root, // Pass current directory as scan_root
        dry_run,
        verbose,
    )?;

    // --- NEW: Generate a sample build.rs using the new composer ---
    let target_build_rs_parts_dir = PathBuf::from("buildrs_parts_for_target_crate");
    let generated_target_build_rs_path = wrapped_workspace_output_dir.join("generated_target_build.rs");
    
    // Ensure the output directory exists
    fs::create_dir_all(&wrapped_workspace_output_dir)
        .context(format!("Failed to create output directory for target build.rs: {}", wrapped_workspace_output_dir.display()))?;

    if verbose {
        println!("Attempting to compose target build.rs from parts in: {}", target_build_rs_parts_dir.display());
    }
    build_script_composer::compose_build_script_from_parts(
        &target_build_rs_parts_dir,
        &generated_target_build_rs_path,
    )?;
    // --- END NEW ---

    // Process each crate for declaration splitting
    if verbose {
        println!("Processing crates for declaration splitting...");
    }
    
    // The previous walkdir iteration and submodule processing logic is removed
    // as generate_wrapped_workspace now handles finding Cargo.toml files within the specified scan_root.
    // However, eager_split_crate is still called for each crate found.
    // For now, let's assume the current directory holds the crate we want to split.
    let current_crate_path = PathBuf::from("./").canonicalize().context("Failed to canonicalize current path for eager splitter")?;
    let lib_rs = current_crate_path.join("src/lib.rs");

    if lib_rs.exists() {
        if verbose {
            println!("  Processing current crate: {}", current_crate_path.display());
        }
        let paths = setup_crate_paths(&current_crate_path)?;
        eager_splitter::eager_split_crate(&paths, &global_config_mut)?; // Use mutable clone here
        if verbose {
            println!("  Finished processing current crate.");
        }
    } else {
        if verbose {
            println!("  No src/lib.rs found in current directory. Skipping eager_split_crate for current crate.");
        }
    }
    
    if verbose {
        println!("\nWrapped workspace generation finished.");
    }
    Ok(())
}

fn run_ecosystem_scan_mode(
    verbose: bool,
    dry_run: bool,
    base_path: &Path,
    recursive: bool,
    global_config: &SplitDeclsConfig,
) -> Result<()> {
    ecosystem_processor::process_ecosystem(verbose, dry_run, base_path, recursive, global_config)
}

fn run_execute_goal_workflow_mode(
    verbose: bool,
    dry_run: bool,
    goal_file: &PathBuf,
    global_config: &SplitDeclsConfig,
) -> Result<()> {
    if verbose {
        if dry_run {
            println!("*** Running in DRY-RUN mode. No files will be modified. ***");
        }
        println!("Executing workflow from: {}", goal_file.display());
    }

    let goal_config = GoalConfig::load_from_file(goal_file)
        .context(format!("Failed to load goal file from {}", goal_file.display()))?;

    let mut workflow_executor = WorkflowExecutor::new(verbose, dry_run, global_config.clone());
    workflow_executor.execute(&goal_config.workflow)?;

    Ok(())
}

fn run_bootstrap_mode(
    verbose: bool,
    dry_run: bool,
    output_dir_override: Option<&PathBuf>,
    global_config: &SplitDeclsConfig,
) -> Result<()> {
    if verbose {
        if dry_run {
            println!("*** Running in DRY-RUN mode. No files will be modified. ***");
        }
        println!("Running bootstrap mode.");
    }

    let wrapped_workspace_output_dir = output_dir_override
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("output2"));

    // Define the workflow programmatically for the build stage
    let build_workflow = Workflow {
        name: "Bootstrap Build Stage".to_string(),
        description: "Builds the generated code in the output directory.".to_string(),
        style_influences: vec![],
        stages: vec![
            split_decls_rs::goal_parser::Stage {
                name: "Build Generated Code".to_string(),
                description: format!("Runs 'cargo build' in the generated workspace at {}.", wrapped_workspace_output_dir.display()),
                processor_hint: None,
                inputs: vec![],
                outputs: vec![
                    split_decls_rs::goal_parser::Output {
                        name: "stdout".to_string(),
                        output_type: "string".to_string(),
                        description: "Standard output of the build command.".to_string(),
                    },
                    split_decls_rs::goal_parser::Output {
                        name: "stderr".to_string(),
                        output_type: "string".to_string(),
                        description: "Standard error of the build command.".to_string(),
                    },
                    split_decls_rs::goal_parser::Output {
                        name: "status".to_string(),
                        output_type: "integer".to_string(),
                        description: "Exit status code of the build command.".to_string(),
                    },
                ],
                operation: split_decls_rs::goal_parser::Operation::Shell(
                    split_decls_rs::goal_parser::ShellCommandOperation {
                        op_type: "shell".to_string(),
                        command: "cargo build --workspace".to_string(),
                        working_dir: Some(wrapped_workspace_output_dir.to_string_lossy().to_string()),
                        capture_output: true,
                        error_on_failure: true,
                    },
                ),
                tasks: vec![],
            },
        ],
    };

    // First, run the wrapped workspace generation directly
    run_wrapped_workspace_mode(verbose, dry_run, output_dir_override, global_config)?;

    // Then, execute the build workflow using the WorkflowExecutor
    let mut workflow_executor = WorkflowExecutor::new(verbose, dry_run, global_config.clone());
    workflow_executor.execute(&build_workflow)?;

    // After the workflow execution, we can check the results, e.g., the build status
    if verbose {
        if let Some(status_value) = workflow_executor.get_context_value("status") {
            if let Some(status) = status_value.as_integer() {
                println!("Build command exited with status: {}", status);
                if status != 0 {
                    println!("Build failed. See 'stderr' in context for details.");
                }
            }
        }
    }

    Ok(())
}

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
    let cli = Cli::parse();

    if cli.verbose {
        println!("Verbose mode enabled.");
        println!("CLI args: {:?}", cli);
    }

    // Load global config - this will be common to both modes
    let workspace_root = PathBuf::from("./");
    let global_config_path = workspace_root.join("split-decls-rs.toml");
    let global_config = if global_config_path.exists() {
        SplitDeclsConfig::load_from_file(&global_config_path)
            .context("Failed to load global split-decls-rs config")?
    } else {
        if cli.verbose {
            println!("No split-decls-rs.toml found at {}, using default configuration.", global_config_path.display());
        }
        SplitDeclsConfig::default()
    };
    if cli.verbose {
        println!("Global config loaded: {:?}", global_config);
    }

    match &cli.command {
        Commands::WrappedWorkspace { output_dir, dry_run } => {
            run_wrapped_workspace_mode(cli.verbose, *dry_run, output_dir.as_ref(), &global_config)?;
        }
        Commands::EcosystemScan { base_path, recursive, dry_run } => {
            run_ecosystem_scan_mode(cli.verbose, *dry_run, base_path, *recursive, &global_config)?;
        }
        Commands::ExecuteGoalWorkflow { goal_file, dry_run } => {
            run_execute_goal_workflow_mode(cli.verbose, *dry_run, goal_file, &global_config)?;
        }
        Commands::Bootstrap { output_dir, dry_run } => {
            run_bootstrap_mode(cli.verbose, *dry_run, output_dir.as_ref(), &global_config)?;
        }
    }

    Ok(())
}
