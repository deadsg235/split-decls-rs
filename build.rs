use cargo_toml_generator_macros::define_root_cargo_toml;
use cargo_toml_generator_types::CargoToml;
use std::{env, fs, path::Path};

mod build_helpers;
use build_helpers::cargo_toml_parts::mkbuildrs;

fn main() {
    // Tell Cargo to rerun this build script if src/main.rs or build.rs changes
    // This is a placeholder, a more robust solution would track macro input files.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/main.rs"); // If main.rs is where the macro is called.

    let generated_cargo_toml: CargoToml = define_root_cargo_toml! { mkbuildrs!() };
}
