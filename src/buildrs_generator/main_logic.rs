use proc_macro2::TokenStream;
use quote::quote;
 // Required for SplitDeclsConfig in the generated build.rs

use syn::LitStr;

pub fn generate_main_logic_token_stream(
    old_lib_rs_path_lit: &LitStr, // Still needed for rerun-if-changed, though not parsed
    old_build_rs_path_lit: &LitStr, // Still needed for rerun-if-changed
    _decls_output_dir_lit: &LitStr, // Still needed for rerun-if-changed
    crate_name_sanitized_lit: &LitStr,
) -> TokenStream {
    quote! {
        fn main() -> Result<()> {
            println!("cargo:rerun-if-changed=build.rs");
            println!("cargo:rerun-if-changed{}", #old_lib_rs_path_lit); // oldlib.rs
            println!("cargo:rerun-if-changed{}", #old_build_rs_path_lit); // oldbuild.rs
            println!("cargo:rerun-if-changed=.split-decls-config.toml");

            // Load configuration for this crate to get patch file paths
            let config_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?)
                .join(".split-decls-config.toml");
            let config = SplitDeclsConfig::load_from_file(&config_path)
                .context(format!("Failed to load config from {}", config_path.display()))?;

            // Watch for changes in patch files
            let current_crate_name_for_patch = #crate_name_sanitized_lit.to_string();
            if let Some(patches_for_crate) = config.patches.get(&current_crate_name_for_patch) {
                for patch_spec in patches_for_crate {
                    let patch_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?).join(&patch_spec.path);
                    println!("cargo:rerun-if-changed={}", patch_path.display());
                }
            }
            
            // This build.rs no longer performs the eager splitting.
            // It only monitors for changes that would require re-running the main split-decls-rs tool.
            println!("cargo:note=build.rs finished. If `split-decls-rs` needs to be re-run, changes will be detected.");

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use syn::LitStr;

    #[test]
    fn test_generate_main_logic_token_stream_basic() {
        let old_lib_rs_path_lit = LitStr::new("/tmp/test_crate/src/oldlib.rs", Span::call_site());
        let old_build_rs_path_lit = LitStr::new("/tmp/test_crate/oldbuild.rs", Span::call_site());
        let decls_output_dir_lit = LitStr::new("/tmp/test_crate/src/decls", Span::call_site());
        let crate_name_sanitized_lit = LitStr::new("test_crate_name", Span::call_site());

        let token_stream = generate_main_logic_token_stream(
            &old_lib_rs_path_lit,
            &old_build_rs_path_lit,
            &decls_output_dir_lit,
            &crate_name_sanitized_lit,
        );

        let code = token_stream.to_string();
        println!("Generated code for main_logic.rs test:\n{}", code);
    }
}
