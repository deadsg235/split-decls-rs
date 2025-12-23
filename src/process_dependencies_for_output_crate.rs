use anyhow::Context;
use toml::Value;
use toml::Table;
use std::path::Path;
use split_decls_types::SplitDeclsConfig;
use crate::generate_wrapped_workspace::path_diff;
//process_dependencies_for_output_crate
// New helper function to process dependencies
pub fn process_dependencies_for_output_crate(
    original_deps: &Table,
    global_config: &SplitDeclsConfig,
    output_dir: &Path,
    project_root: &Path, // This will be the original scan_root
    verbose: bool,
) -> anyhow::Result<Table> {
    let mut new_deps = Table::new();
    for (dep_name, dep_value) in original_deps.iter() {
        println!("DEBUG: Processing dependency: {}", dep_name);
        if let Some(dep_table) = dep_value.as_table() {
            println!("DEBUG:   Is a table dependency.");
            let mut processed_dep_table = dep_table.clone();
            
            // Handle workspace dependencies
            if dep_table.contains_key("workspace") && dep_table["workspace"].as_bool().unwrap_or(false) {
                println!("DEBUG:   Is a workspace dependency.");
                if let Some(resolved_dep_info) = global_config.workspace_dependencies.get(dep_name) {
                    println!("DEBUG:     Found in global_config.workspace_dependencies: {:?}", resolved_dep_info);
                    if let Some(resolved_path_str) = resolved_dep_info.get("path").and_then(|v| v.as_str()) {
                        println!("DEBUG:       Resolved as path: {}", resolved_path_str);
                        let absolute_resolved_path = project_root.join(resolved_path_str);
                        let relative_path = path_diff(output_dir, &absolute_resolved_path)
                            .context(format!("Failed to calculate relative path for workspace dep '{}'", dep_name))?;
                        processed_dep_table.insert("path".to_string(), Value::String(relative_path.display().to_string()));
                        processed_dep_table.remove("workspace"); // Remove workspace key
                        // Remove version if it exists and path is used
                        processed_dep_table.remove("version");
                    }// path string
                    // If no path, try to resolve version
                    else if let Some(resolved_version_str) = resolved_dep_info.get("version").and_then(|v| v.as_str()) {
                        processed_dep_table.insert("version".to_string(), Value::String(resolved_version_str.to_string()));
                        processed_dep_table.remove("workspace"); // Remove workspace key
                    }
                    else { // This `else` handles case where no path or version found in resolved_dep_info
                        if verbose {
                            println!("Warning: No path or version found for workspace dependency '{}' in global_config. Keeping original.", dep_name);
                        }
                    }
                } else {
                    if verbose {
                        println!("Warning: Workspace dependency '{}' not found in global_config. Keeping original.", dep_name);
                    }
                }
            } 
            // Handle regular path dependencies
            else if let Some(original_path_value) = dep_table.get("path") {
                if let Some(original_path_str) = original_path_value.as_str() {
                    let absolute_original_path = project_root.join(original_path_str);
                    let relative_path = path_diff(output_dir, &absolute_original_path)
                        .context(format!("Failed to calculate relative path for path dep '{}'", dep_name))?;
                    processed_dep_table.insert("path".to_string(), Value::String(relative_path.display().to_string()));
                }
            }

            new_deps.insert(dep_name.clone(), Value::Table(processed_dep_table));

        } else {
            // Non-table dependency (e.g., version string directly)
            new_deps.insert(dep_name.clone(), dep_value.clone());
        }
    }
    Ok(new_deps)
}
