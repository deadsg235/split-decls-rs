use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Defines a single string replacement operation.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct StringReplacement {
    pub old: String,
    pub new: String,
}

/// Defines a single patch file and an optional Git reference for context.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PatchSpec {
    /// Path to the patch .rs file, relative to the workspace root.
    pub path: PathBuf,
    /// Optional Git reference (branch, tag, commit hash) associated with this patch.
    /// This indicates the state of the repository for which this patch is relevant.
    pub git_reference: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)] // Added Clone for easier handling in main.rs
pub struct SplitDeclsConfig {
    /// A list of names for active overlay modules or features.
    /// This can be used to conditionally enable/disable logic in the generated build.rs.
    pub active_overlay_modules: Vec<String>,
    /// Raw Rust code to be injected as a prelude/header into each generated declaration file.
    pub custom_prelude_overlay: Option<String>,
    /// Optional path to the rustc source tree that should be processed.
    pub rustc_source_path: Option<PathBuf>,
    /// A map of crate names to a list of patch specifications.
    pub patches: HashMap<String, Vec<PatchSpec>>,
    /// A list of raw string replacements to apply to oldlib.rs content before AST parsing.
    pub string_replacements: Option<Vec<StringReplacement>>,
    /// A map of crate names to local paths for [patch.crates-io] entries.
    pub crates_io_patches: HashMap<String, PathBuf>,
    /// Optional GitHub organization to use for forking upstream repositories.
    pub github_org: Option<String>,
    /// Default branches to apply patches to for Git repositories.
    #[serde(default)]
    pub default_branches_to_patch: Vec<String>,
    /// Explicit mapping from upstream repository URLs to their fork URLs.
    #[serde(default)]
    pub repo_fork_mapping: HashMap<String, String>,
    /// A map of all workspace dependencies (from [workspace.dependencies])
    /// to their TOML Value representation.
    #[serde(default)]
    pub workspace_dependencies: HashMap<String, toml::Value>,
}

impl SplitDeclsConfig {
    pub fn load_from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        if !path.exists() {
            println!("No split-decls-rs.toml found at {}, using default configuration.", path.display());
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
}