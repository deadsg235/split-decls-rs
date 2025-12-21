pub fn process_crate(crate_path: &Path, global_config: &SplitDeclsConfig, dry_run: bool) -> Result<()> {
    let paths = setup_crate_paths(crate_path)?;
    println!("\n=== Processing crate: {} ===", paths.crate_name);

    // Create a crate-specific config for writing to .split-decls-config.toml
    let mut crate_config = SplitDeclsConfig::default();
    crate_config.active_overlay_modules = global_config.active_overlay_modules.clone();
    crate_config.custom_prelude_overlay = global_config.custom_prelude_overlay.clone();
    crate_config.string_replacements = global_config.string_replacements.clone();
    crate_config.crates_io_patches = global_config.crates_io_patches.clone(); // Copy new field

    // Filter patches relevant to this crate from the global config
    let crate_name_str = paths.crate_name.clone();
    if let Some(global_patches_map) = &global_config.patches {
        if let Some(crate_patches) = global_patches_map.get(&crate_name_str) {
            crate_config.patches.get_or_insert_with(HashMap::new).insert(crate_name_str.clone(), crate_patches.clone());
        }
    }
    // Note: rustc_source_path is not copied here, as it's a global setting for the tool,
    // not something needed by the generated build.rs

    let serialized_config = toml::to_string(&crate_config)
        .context("Failed to serialize SplitDeclsConfig")?;
    // Write the crate-specific config always, as it's an internal file for the build.rs
    fs::write(&paths.target_config_path, serialized_config)
        .context(format!("Failed to write .split-decls-config.toml to {}", paths.target_config_path.display()))?;
    println!("Wrote config to {}", paths.target_config_path.display());

    backup_original_cargotoml(&paths, dry_run)?;
    backup_original_files(&paths, dry_run)?;
    generate_new_cargotoml(&paths, global_config, &paths.crate_path, dry_run)?;
    generate_new_lib_rs(&paths, dry_run)?;
    // generate_new_build_rs(&paths, dry_run)?; // We will modify this or remove it later

    // --- Eager Splitting Logic ---
    let mut old_lib_rs_content = fs::read_to_string(&paths.old_lib_rs_path)
        .context("Failed to read oldlib.rs content for eager splitting")?;

    if let Some(replacements) = &global_config.string_replacements {
        for sr in replacements {
            old_lib_rs_content = old_lib_rs_content.replace(&sr.old, &sr.new);
            println!("Applied string replacement: '{}' -> '{}'", sr.old, sr.new);
        }
    }

    let mut syntax_tree = syn::parse_file(&old_lib_rs_content)
        .context("Failed to parse oldlib.rs content for eager splitting")?;

    buildrs_ast_utils::apply_patches_to_syntax_tree(
        &mut syntax_tree,
        &paths.crate_name.replace("-", "_"),
        &crate_config, // Use crate_config which has filtered patches
    )?;

    eager_splitter::split_and_generate_decls(
        &syntax_tree,
        &paths,
        &crate_config,
        dry_run,
    )?;
    // --- End Eager Splitting Logic ---

    // Now generate a simplified build.rs that handles dynamic patching if needed
    generate_new_build_rs(&paths, dry_run)?;

    Ok(())
}
