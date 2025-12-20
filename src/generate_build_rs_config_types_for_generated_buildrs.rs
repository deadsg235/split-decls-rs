use anyhow::{Context, Result};
use proc_macro2::TokenStream;
use quote::quote;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use toml;

pub fn generate_build_rs_config_types_for_generated_buildrs() -> TokenStream {
    quote! {
        // Configuration structs (re-defined for standalone build.rs)
        #[derive(Debug, Default, Serialize, Deserialize, Clone)]
        pub struct PatchSpec {
            pub path: PathBuf,
            pub git_reference: Option<String>,
        }

        #[derive(Debug, Default, Serialize, Deserialize, Clone)]
        pub struct StringReplacement {
            pub old: String,
            pub new: String,
        }

        #[derive(Debug, Default, Serialize, Deserialize, Clone)]
        pub struct SplitDeclsConfig {
            pub active_overlay_modules: Vec<String>,
            pub custom_prelude_overlay: Option<String>,
            pub rustc_source_path: Option<PathBuf>,
            pub patches: HashMap<String, Vec<PatchSpec>>,
            pub string_replacements: Option<Vec<StringReplacement>>,
        }

        impl SplitDeclsConfig {
            pub fn load_from_file(path: &Path) -> anyhow::Result<Self> {
                if !path.exists() {
                    return Ok(Self::default());
                }
                let content = std::fs::read_to_string(path)?;
                let config: Self = toml::from_str(&content)?;
                Ok(config)
            }
        }
    }
}