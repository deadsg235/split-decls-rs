use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

/// Encapsulates all relevant file paths for a target crate.
pub struct CratePaths {
    pub crate_path: PathBuf,
    pub crate_name: String,
    pub lib_rs_path: PathBuf,

    pub build_rs_path: PathBuf,

    pub cargo_toml_path: PathBuf, // New field for Cargo.toml
    pub old_cargo_toml_path: PathBuf, // Backup path for original Cargo.toml

    pub decls_output_dir: PathBuf,
    pub target_config_path: PathBuf,
    pub output_crate_path: PathBuf,
}

/// Sets up and returns all relevant file paths for a given crate.
pub fn setup_crate_paths(crate_path: &Path) -> Result<CratePaths> {
    let crate_name_os_str = crate_path
        .file_name()
        .context("Crate path has no file name")?;
    let crate_name = crate_name_os_str
        .to_str()
        .context("Crate name is not valid UTF-8")?;

    let current_dir = std::env::current_dir()?;
    let relative_crate_path = crate_path.strip_prefix(&current_dir)
        .unwrap_or(crate_path); // Fallback if not within current_dir

    let output_crate_path = PathBuf::from("output2").join(relative_crate_path);

    let lib_rs_path = crate_path.join("src").join("lib.rs");

    let build_rs_path = crate_path.join("build.rs");

    let cargo_toml_path = crate_path.join("Cargo.toml");
    let old_cargo_toml_path = crate_path.join("oldCargo.toml");

    let decls_output_dir = output_crate_path.join("src").join("decls"); // Decls within the output crate
    let target_config_path = output_crate_path.join(".split-decls-config.toml"); // Config within the output crate

    Ok(CratePaths {
        crate_path: crate_path.to_path_buf(),
        crate_name: crate_name.to_string(),
        lib_rs_path,

        build_rs_path,

        cargo_toml_path,
        old_cargo_toml_path,

        decls_output_dir,
        target_config_path,
        output_crate_path,
    })
}