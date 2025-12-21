/// Generates the new Cargo.toml for the crate, adding necessary build-dependencies.
pub fn generate_new_cargotoml(paths: &CratePaths, global_config: &SplitDeclsConfig, original_crate_real_path: &Path, dry_run: bool) -> Result<()> {
    // List of dependencies that should use `workspace = true`
    const RUNTIME_WORKSPACE_DEPS: &[&str] = &[
        "proc-macro2",
        "quote",
        "syn",
    ];

    const BUILD_WORKSPACE_DEPS: &[&str] = &[
        "anyhow",
        "proc-macro2",
        "quote",
        "syn",
        "introspector_decl2_macros",
        "introspector_decl_core",
        "introspector_macro_helpers",
        "introspector_decl_common",
        "serde", // Added for build script
        "toml",    // Added for build script
    ];

    let mut cargo_toml_content = fs::read_to_string(&paths.old_cargo_toml_path)
        .context(format!("Failed to read old Cargo.toml from {}", paths.old_cargo_toml_path.display()))?;
    
    // If the file was empty (no Cargo.toml existed), initialize with a minimal structure
    if cargo_toml_content.trim().is_empty() {
        cargo_toml_content = format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            paths.crate_name
        );
    }

    let mut cargo_toml: CargoToml = toml::from_str(&cargo_toml_content)
        .context(format!("Failed to parse old Cargo.toml from {}", paths.old_cargo_toml_path.display()))?;

    // Remove unwanted top-level sections for submodules
    cargo_toml.other.remove("workspace");
    cargo_toml.other.remove("profile"); // This removes a top-level `[profile]` section
    cargo_toml.other.remove("lints"); // This removes a top-level `[lints]` section
    cargo_toml.other.remove("bench"); // Remove top-level `[bench]` sections

    // Iterate through `other` and remove any keys starting with "profile.", "lints.", "bench."
    // or containing "workspace" (unless it's specifically "package.workspace" which is handled in Package struct)
    let keys_to_remove: Vec<String> = cargo_toml.other.keys()
        .filter(|k| 
            k.starts_with("profile.") || 
            k.starts_with("lints.") || 
            k.starts_with("bench.") ||
            (k.contains("workspace") && *k != "workspace") // Remove other workspace-related keys
        )
        .cloned()
        .collect();

    for key in keys_to_remove {
        cargo_toml.other.remove(&key);
    }

    // Helper to ensure a dependency uses workspace = true and specified features
    let ensure_workspace_dependency = |table: &mut toml::Table, dep_name: &str, features: Option<Vec<&str>>| {
        let mut dep_table_value = toml::Table::new();
        dep_table_value.insert("workspace".to_string(), toml::Value::Boolean(true));
        if let Some(feats) = features {
            let features_array = toml::Value::Array(feats.into_iter().map(|f| toml::Value::String(f.to_string())).collect());
            dep_table_value.insert("features".to_string(), features_array);
        }
        table.insert(dep_name.to_string(), toml::Value::Table(dep_table_value));
    };

    // Process [dependencies] to ensure RUNTIME_WORKSPACE_DEPS are present and convert path-deps to workspace = true
    let deps_table = &mut cargo_toml.dependencies;
    process_dependency_table(deps_table, global_config, original_crate_real_path)?; // Apply general dependency processing
    for dep_name in RUNTIME_WORKSPACE_DEPS {
        match *dep_name {
            "syn" => ensure_workspace_dependency(deps_table, dep_name, Some(vec!["full"])),
            _ => ensure_workspace_dependency(deps_table, dep_name, None),
        }
    }

    // Process [dev-dependencies] (existing logic remains)
    let dev_deps_table = &mut cargo_toml.dev_dependencies;
    process_dependency_table(dev_deps_table, global_config, original_crate_real_path)?; // Apply general dependency processing
    for dep_name in RUNTIME_WORKSPACE_DEPS { // Also update if runtime deps exist in dev-deps
        if dev_deps_table.contains_key(*dep_name) {
            ensure_workspace_dependency(dev_deps_table, dep_name, None); // Added None for features
        }
    }

    // Process [build-dependencies] to ensure BUILD_WORKSPACE_DEPS are present
    let build_deps_table = &mut cargo_toml.build_dependencies;
    process_dependency_table(build_deps_table, global_config, original_crate_real_path)?; // Apply general dependency processing
    for dep_name in BUILD_WORKSPACE_DEPS {
        match *dep_name {
            "syn" => ensure_workspace_dependency(build_deps_table, dep_name, Some(vec!["full", "visit"])),
            "serde" => ensure_workspace_dependency(build_deps_table, dep_name, Some(vec!["derive"])),
            _ => ensure_workspace_dependency(build_deps_table, dep_name, None),
        }
    }

    // Apply [patch.crates-io] entries
    if let Some(crates_io_patches_map) = &global_config.crates_io_patches {
        if !crates_io_patches_map.is_empty() {
            let mut crates_io_table = toml::Table::new();
            for (crate_name, path) in crates_io_patches_map {
            crates_io_table.insert(
                crate_name.clone(),
                toml::Table::from_iter([(
                    "path".to_string(),
                    toml::Value::String(path.to_str().context("Path not valid UTF-8")?.to_string()),
                )])
                .into(),
            );
        }
        cargo_toml.patch.insert("crates-io".to_string(), crates_io_table.into());
    }

    let new_cargo_toml_content = toml::to_string(&cargo_toml)
        .context("Failed to serialize new Cargo.toml")?;
    
    if dry_run {
        let new_path = paths.cargo_toml_path.with_extension("new"); // Changed to .new
        fs::write(&new_path, new_cargo_toml_content)
            .context(format!("Failed to write new Cargo.toml to {}", new_path.display()))?;
        println!("Dry-run: Generated new Cargo.toml content to {} for crate {}", new_path.display(), paths.crate_name);
    } else {
        fs::write(&paths.cargo_toml_path, new_cargo_toml_content)
            .context(format!("Failed to write new Cargo.toml to {}", paths.cargo_toml_path.display()))?;
        println!("Generated new Cargo.toml for crate {}", paths.crate_name);
    }

    Ok(())
}
