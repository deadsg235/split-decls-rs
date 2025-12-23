 use serde :: { Deserialize , Serialize } ; use std :: collections :: HashMap ; # [test] fn test_apply_workspace_package_defaults_to_root () -> Result < () > { let dir = tempdir () ? ; let root_cargo_toml_path = dir . path () . join ("Cargo.toml") ; let initial_content = r#"
[workspace.package]
edition = "2024"
version = "1.0.0"
authors = ["Workspace Author"]
description = "Default workspace description"
homepage = "https://example.com"

[package]
name = "my-root-crate"
version = "0.9.0" # This should not be overwritten

[workspace]
members = ["crate_a"]
"# ; fs :: write (& root_cargo_toml_path , initial_content) ? ; apply_workspace_package_defaults_to_root (& root_cargo_toml_path , false , false) ? ; let modified_content = fs :: read_to_string (& root_cargo_toml_path) ? ; println ! ("{}" , modified_content) ; assert ! (modified_content . contains ("name = \"my-root-crate\"")) ; assert ! (modified_content . contains ("version = \"0.9.0\"")) ; assert ! (modified_content . contains ("edition = \"2024\"")) ; assert ! (modified_content . contains ("authors = [\"Workspace Author\"]")) ; assert ! (modified_content . contains ("description = \"Default workspace description\"")) ; assert ! (modified_content . contains ("homepage = \"https://example.com\"")) ; assert ! (modified_content . contains ("[workspace.package]")) ; assert ! (modified_content . contains ("edition = \"2024\"")) ; assert ! (modified_content . contains ("version = \"1.0.0\"")) ; assert ! (modified_content . contains ("authors = [\"Workspace Author\"]")) ; Ok (()) }