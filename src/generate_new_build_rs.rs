use std::fs;
use anyhow::{Context, Result};
use crate::paths::CratePaths; use crate::buildrs_generator;
use crate::add_generated_rust_header;

/// Generates the new build.rs for the crate.
pub fn generate_new_build_rs(paths: &CratePaths, dry_run: bool) -> Result<()> {
    // Generate the TokenStream for the build.rs content using the generator module
    let build_rs_token_stream = buildrs_generator::generate_build_rs_token_stream(
        &paths.decls_output_dir,
        &paths.crate_name.replace("-", "_"),
    )?;

    if dry_run {
        let new_path = paths.build_rs_path.with_extension("new");
        add_generated_rust_header!(
            &new_path,
            build_rs_token_stream.to_string().as_str(),
            file!(),
            line!()
        )
        .context(format!("Failed to write new build.rs to {}", new_path.display()))?;
        println!("Dry-run: Generated new build.rs content to {} for crate {}", new_path.display(), paths.crate_name);
    } else {
        add_generated_rust_header!(
            &paths.build_rs_path,
            build_rs_token_stream.to_string().as_str(),
            file!(),
            line!()
        )
        .context(format!("Failed to write new build.rs to {}", paths.build_rs_path.display()))?;
        println!("Generated build.rs for crate {}", paths.crate_name);
    }
    Ok(())
}
