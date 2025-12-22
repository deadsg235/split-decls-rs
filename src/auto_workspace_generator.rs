use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;
use toml::Value;

pub fn generate_workspace_deps_from_project_root(project_root: &Path) -> Result<(Vec<String>, Vec<String>)> {
    println!("DEBUG: generate_workspace_deps_from_project_root called with: {}", project_root.display());
    let mut workspace_deps = Vec::new();
    let mut patch_entries = Vec::new();
    
    // Find all Cargo.toml files in project root
    let cargo_tomls = find_cargo_tomls(project_root)?;
    
    for cargo_path in cargo_tomls {
        if let Some(crate_info) = extract_crate_info(&cargo_path)? {
            let relative_path = cargo_path.parent().unwrap()
                .strip_prefix(project_root)?
                .to_string_lossy();
            
            // Generate workspace dependency
            let dep_entry = format!(
                "{} = {{ path = \"{}\" }}", 
                crate_info.name, 
                relative_path
            );
            workspace_deps.push(dep_entry.clone());
            
            // Generate patch entry
            patch_entries.push(dep_entry);
        }
    }
    
    Ok((workspace_deps, patch_entries))
}

fn find_cargo_tomls(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut cargo_tomls = Vec::new();
    
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.file_name() == Some("Cargo.toml".as_ref()) {
                cargo_tomls.push(path);
            } else if path.is_dir() && !should_skip_dir(&path) {
                cargo_tomls.extend(find_cargo_tomls(&path)?);
            }
        }
    }
    
    Ok(cargo_tomls)
}

fn should_skip_dir(path: &Path) -> bool {
    let name = path.file_name().unwrap().to_string_lossy();
    matches!(name.as_ref(), "target" | ".git" | "node_modules")
}

struct CrateInfo {
    name: String,
}

fn extract_crate_info(cargo_path: &Path) -> Result<Option<CrateInfo>> {
    let content = fs::read_to_string(cargo_path)?;
    let toml: Value = toml::from_str(&content)?;
    
    if let Some(package) = toml.get("package") {
        if let Some(name) = package.get("name").and_then(|n| n.as_str()) {
            return Ok(Some(CrateInfo {
                name: name.to_string(),
            }));
        }
    }
    
    Ok(None)
}
