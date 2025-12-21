pub fn process_crates_in_path(
    root_path: &Path,
    current_crate_name: &str,
    global_config: &SplitDeclsConfig,
    is_rustc_source: bool, // New parameter to control skipping logic
    dry_run: bool,
) -> Result<()> {
    for entry in WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok()) {
        if entry.file_name() == "Cargo.toml" {
            let cargotoml_path = entry.path();
            let crate_path = cargotoml_path
                .parent()
                .context("Cargo.toml has no parent directory")?;

            // Read Cargo.toml to check if it's a virtual manifest
            let cargo_toml_content = fs::read_to_string(&cargotoml_path)
                .context(format!("Failed to read Cargo.toml from {}", cargotoml_path.display()))?;
            
            #[derive(Debug, serde::Deserialize)]
            struct MinimalCargoToml {
                package: Option<toml::Table>,
            }
            let minimal_cargo_toml: MinimalCargoToml = toml::from_str(&cargo_toml_content)
                .context(format!("Failed to parse Cargo.toml from {}", cargotoml_path.display()))?;

            // Skip virtual manifests (Cargo.toml without a [package] section)
            if minimal_cargo_toml.package.is_none() {
                println!("Skipping virtual manifest: {}", cargotoml_path.display());
                continue;
            }

            let crate_name = crate_path
                .file_name()
                .and_then(|s| s.to_str())
                .context("Could not get crate name")?;

            // Always skip self crate
            if crate_name == current_crate_name {
                println!("Skipping self crate: {}", current_crate_name);
                continue;
            }

            if !is_rustc_source {
                // Apply original skipping logic only for non-rustc source
                if crate_path.starts_with(Path::new("submodules/rust")) {
                    println!("Skipping rust submodule crate: {}", crate_name);
                    continue;
                }
                if crate_path.starts_with(Path::new("submodules/rust-analyzer/")) {
                    println!("Skipping rust-analyzer crate: {}", crate_name);
                    continue;
                }
                if crate_path.starts_with(Path::new("crates/trait-fixer")) {
                    println!("Skipping trait-fixer crate: {}", crate_name);
                    continue;
                }
                if crate_path.starts_with(Path::new("tools")) {
                    println!("Skipping tools crate: {}", crate_name);
                    continue;
                }
            } else {
                // For rustc source, skip the top-level virtual manifest if it exists
                if cargotoml_path == root_path.join("Cargo.toml") {
                    println!("Skipping root Cargo.toml of rustc source: {}", cargotoml_path.display());
                    continue;
                }
            }

            process_crate(crate_path, global_config, dry_run)?;
        }
    }
    Ok(())
}

