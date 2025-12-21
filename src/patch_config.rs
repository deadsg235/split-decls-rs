use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct PatchTarget {
    pub name: String,
    pub path: PathBuf, // Local path to the crate
    pub repo_url: Option<String>, // Upstream Git repository URL
    pub git_reference: Option<String>, // Git reference (branch, tag, commit) to checkout
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PatchedDependency {
    pub name: String,
    pub version: Option<String>,
    pub path: Option<PathBuf>,
    pub features: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PatchConfig {
    pub targets: Vec<PatchTarget>,
    #[serde(default)]
    pub patched_dependencies: Vec<PatchedDependency>,
}

impl PatchConfig {
    pub fn load_from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        if !path.exists() {
            anyhow::bail!("Patch config file not found at {}", path.display());
        }
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
}
