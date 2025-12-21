use std::path::{PathBuf, Path};
use anyhow::{Result, Context};

pub struct CliArgs {
    pub dry_run: bool,
    pub verbose: bool,
    pub wrapped_workspace_output_dir: PathBuf,
    pub patch_config_path_str: String,
    pub generate_wrapped_workspace_mode: bool, 
}

pub fn parse_args() -> Result<CliArgs> {
    let mut args: Vec<String> = std::env::args().collect();

    let dry_run_index = args.iter().position(|arg| arg == "--dry-run");
    let dry_run = dry_run_index.is_some();
    if let Some(index) = dry_run_index {
        args.remove(index);
    }
    
    let verbose_index = args.iter().position(|arg| arg == "--verbose");
    let verbose = verbose_index.is_some();
    if let Some(index) = verbose_index {
        args.remove(index);
    }

    let mut wrapped_workspace_output_dir = PathBuf::from("output");
    let mut generate_wrapped_workspace_mode = true; 

    // Check for --no-wrapped-workspace argument to disable wrapped workspace generation
    if let Some(index) = args.iter().position(|arg| arg == "--no-wrapped-workspace") { 
        generate_wrapped_workspace_mode = false;
        args.remove(index);
    }

    // Check for --output-dir argument to override the default output directory
    if let Some(index) = args.iter().position(|arg| arg == "--output-dir") { 
        args.remove(index);
        if let Some(output_path_str) = args.get(index) {
            wrapped_workspace_output_dir = PathBuf::from(output_path_str);
            args.remove(index);
        } else {
            anyhow::bail!("--output-dir requires a path.");
        }
    }
    // If --generate-wrapped-workspace is still used, it means an output dir was provided
    // and it should behave like --output-dir, but also explicitly enable wrapped generation mode
    // This is handled by --output-dir now, so --generate-wrapped-workspace can effectively be removed
    // from argument parsing here, as --output-dir and generate_wrapped_workspace_mode cover its intent.
    // However, to allow backwards compatibility or explicit intent, we can keep a check here
    // that if it's present, it ensures generation_mode is true and potentially sets output_dir.
    if let Some(index) = args.iter().position(|arg| arg == "--generate-wrapped-workspace") {
        generate_wrapped_workspace_mode = true; // Explicitly ensure it's active
        args.remove(index); // Remove the flag itself
        if let Some(output_path_str) = args.get(index) {
            wrapped_workspace_output_dir = PathBuf::from(output_path_str);
            args.remove(index); // Remove the path argument
        } else {
            // If the flag is present without a path, assume "output" as default,
            // which is already done by initialization, but ensure mode is true.
        }
    }


    let patch_config_path_str = args.get(1).map_or("patch.toml".to_string(), |s| s.to_string());

    Ok(CliArgs {
        dry_run,
        verbose,
        wrapped_workspace_output_dir,
        patch_config_path_str,
        generate_wrapped_workspace_mode,
    })
}
