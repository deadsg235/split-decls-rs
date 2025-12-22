use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

// --- Top-level Cargo.toml structure ---
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CargoToml {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<Package>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<Workspace>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub dependencies: HashMap<String, Dependency>,
    #[serde(rename = "dev-dependencies", default, skip_serializing_if = "HashMap::is_empty")]
    pub dev_dependencies: HashMap<String, Dependency>,
    #[serde(rename = "build-dependencies", default, skip_serializing_if = "HashMap::is_empty")]
    pub build_dependencies: HashMap<String, Dependency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<PatchSection>,
    #[serde(rename = "workspace.dependencies", default, skip_serializing_if = "HashMap::is_empty")]
    pub workspace_dependencies: HashMap<String, Dependency>,
    // #[serde(rename = "workspace.lints", skip_serializing_if = "Option::is_none")]
    // pub workspace_lints: Option<HashMap<String, String>>, // Assuming simple key-value for now
}

// --- [package] section ---
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Package {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(rename = "rust-version", skip_serializing_if = "Option::is_none")]
    pub rust_version: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolver: Option<String>,
}

// --- [workspace] section ---
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workspace {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolver: Option<String>,
    #[serde(rename = "package", skip_serializing_if = "Option::is_none")]
    pub package_config: Option<Package>, // Shared package settings for workspace members
    #[serde(rename = "dependencies", default, skip_serializing_if = "HashMap::is_empty")]
    pub workspace_dependencies: HashMap<String, Dependency>, // Corresponds to [workspace.dependencies]
}

// --- Dependency structure ---
// This struct tries to be flexible enough to represent different ways to define a dependency.
// Cargo's TOML structure for dependencies can be complex (simple version string, table with path/version/features, etc.)
#[derive(Debug, Clone, PartialEq)]
pub enum Dependency {
    Version(String),
    Table(DependencyTable),
}

impl Serialize for Dependency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Dependency::Version(s) if s.is_empty() => serializer.serialize_str("*"),
            Dependency::Version(s) => serializer.serialize_str(s),
            Dependency::Table(table) => table.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Dependency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let value = toml::Value::deserialize(deserializer)?;
        match value {
            toml::Value::String(s) => Ok(Dependency::Version(s)),
            toml::Value::Table(_) => {
                let table = DependencyTable::deserialize(value).map_err(D::Error::custom)?;
                Ok(Dependency::Table(table))
            }
            _ => Err(D::Error::custom("Invalid dependency format")),
        }
    }
}

impl Default for Dependency {
    fn default() -> Self {
        Dependency::Version(String::new())
    }
}

// Helper struct for the table form of a dependency
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DependencyTable {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_features: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>, // Renames the dependency
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<bool>, // e.g., `dep = { workspace = true }`
}

// --- [patch] section ---
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchSection {
    #[serde(rename = "crates-io", default, skip_serializing_if = "HashMap::is_empty")]
    pub crates_io: HashMap<String, Dependency>,
    // Add other patch sources (e.g., git) if needed
}
