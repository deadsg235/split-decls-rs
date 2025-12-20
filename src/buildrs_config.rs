use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use serde::{Deserialize, Serialize};
use toml;

/// Defines a single patch file and an optional Git reference for context.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PatchSpecBuildrs {
    /// Path to the patch .rs file, relative to the workspace root.
    pub path: PathBuf,
    /// Optional Git reference (branch, tag, commit hash) associated with this patch.
    /// This indicates the state of the repository for which this patch is relevant.
    pub git_reference: Option<String>,
}

/// Defines a single string replacement operation.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct StringReplacementBuildrs {
    pub old: String,
    pub new: String,
}

/// Configuration for split-decls-rs, defined here for the generated build.rs.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SplitDeclsConfigBuildrs {
    pub active_overlay_modules: Vec<String>,
    pub custom_prelude_overlay: Option<String>,
    pub rustc_source_path: Option<PathBuf>,
    pub patches: HashMap<String, Vec<PatchSpecBuildrs>>,
    pub string_replacements: Option<Vec<StringReplacementBuildrs>>,
}

impl SplitDeclsConfigBuildrs {
    pub fn load_from_file(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
}
