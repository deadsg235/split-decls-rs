pub use decls :: * ; # [cfg (feature = "introspector_decl2_macros")] pub use introspector_decl2_macros :: * ; prelude ! { } # [decl_ tool_refactorings_decls_cargo_toml_patcher_tests] # [cfg (test)] mod tests { use super :: * ; use crate :: cargo_toml_patcher_config :: DefaultCargoTomlPatcherConfig ; use tempfile :: tempdir ; # [test] fn test_patch_cargo_toml_standardize_metadata () { let dir = tempdir () . unwrap () ; let cargo_toml_path = dir . path () . join ("Cargo.toml") ; fs :: write (& cargo_toml_path , r#"[package]
name = "test-crate"
version = "0.1.0"

[dependencies]
cargo_metadata = "0.18.1"
"#) . unwrap () ; let config = DefaultCargoTomlPatcherConfig ; let patcher = DefaultCargoTomlPatcher :: new (config) ; let result = patcher . patch_cargo_toml (& cargo_toml_path , false) ; assert ! (result . is_ok ()) ; let content = fs :: read_to_string (& cargo_toml_path) . unwrap () ; let doc = content . parse :: < toml_edit :: DocumentMut > () . unwrap () ; assert_eq ! (doc ["dependencies"] ["cargo_metadata"] . to_string () . trim () , "{ workspace = true, optional = true }") ; } # [test] fn test_patch_cargo_toml_add_external_dep () { let dir = tempdir () . unwrap () ; let cargo_toml_path = dir . path () . join ("Cargo.toml") ; fs :: write (& cargo_toml_path , r#"[package]
name = "test-crate"
version = "0.1.0"

[dependencies]

[features]
"#) . unwrap () ; let config = DefaultCargoTomlPatcherConfig ; let patcher = DefaultCargoTomlPatcher :: new (config) ; let result = patcher . patch_cargo_toml (& cargo_toml_path , false) ; assert ! (result . is_ok ()) ; let content = fs :: read_to_string (& cargo_toml_path) . unwrap () ; let doc = content . parse :: < toml_edit :: DocumentMut > () . unwrap () ; assert_eq ! (doc ["dependencies"] ["anyhow"] . to_string () . trim () , "{ workspace = true, optional = true }") ; assert_eq ! (doc ["features"] ["anyhow_enabled"] . to_string () . trim () , "[\"dep:anyhow\"]") ; } # [test] fn test_patch_cargo_toml_add_local_dep () { let dir = tempdir () . unwrap () ; let cargo_toml_path = dir . path () . join ("Cargo.toml") ; fs :: write (& cargo_toml_path , r#"[package]
name = "test-crate"
version = "0.1.0"

[dependencies]

[features]
"#) . unwrap () ; let config = DefaultCargoTomlPatcherConfig ; let patcher = DefaultCargoTomlPatcher :: new (config) ; let result = patcher . patch_cargo_toml (& cargo_toml_path , false) ; assert ! (result . is_ok ()) ; let content = fs :: read_to_string (& cargo_toml_path) . unwrap () ; let doc = content . parse :: < toml_edit :: DocumentMut > () . unwrap () ; assert_eq ! (doc ["dependencies"] ["cargo-edit-lib"] . to_string () . trim () , "{ path = \"../cargo-edit-lib\", optional = true }") ; assert_eq ! (doc ["features"] ["cargo_edit_lib_enabled"] . to_string () . trim () , "[\"dep:cargo-edit-lib\"]") ; } # [test] fn test_patch_cargo_toml_dry_run () { let dir = tempdir () . unwrap () ; let cargo_toml_path = dir . path () . join ("Cargo.toml") ; fs :: write (& cargo_toml_path , r#"[package]
name = "test-crate"
version = "0.1.0"
"#) . unwrap () ; let config = DefaultCargoTomlPatcherConfig ; let patcher = DefaultCargoTomlPatcher :: new (config) ; let result = patcher . patch_cargo_toml (& cargo_toml_path , true) ; assert ! (result . is_ok ()) ; assert ! (result . unwrap () . contains ("DRY-RUN")) ; let content = fs :: read_to_string (& cargo_toml_path) . unwrap () ; assert ! (! content . contains ("[dependencies.anyhow]")) ; } }