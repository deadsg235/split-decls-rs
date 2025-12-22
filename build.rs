// Use mod with path to directly include code from build_src/
#[path = "build_src/example_module.rs"]
mod example_module;

#[path = "build_src/os.rs"]
mod os_specific_logic;

// Add more modules here as needed:
// #[path = "build_src/another_module.rs"]
// mod another_module;


fn main() {
    // Tell Cargo to rerun this build script if build.rs or any file in build_src changes.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build_src/"); // Watch the entire build_src directory
    println!("cargo:rerun-if-changed=build_src/os/"); // Watch the OS-specific subdirectory

    // Call the run function from the included example_module
    example_module::run();

    // Call the run function from the OS-specific logic module
    os_specific_logic::run();

    // Re-introduce the original Cargo.toml generation logic,
    // assuming build_helpers and other necessary modules are available or will be made available.
    // This part will require the original context of `build.rs` prior to my modifications.
    // For now, I will keep it commented out to ensure the modularity works.
    // If the user wants to re-integrate the Cargo.toml generation, I will need to place it
    // within a module in `build_src` or adapt it directly here.

    // placeholder for original build.rs logic that generates Cargo.toml
    use cargo_toml_generator_macros::define_root_cargo_toml;
    use cargo_toml_generator_types::CargoToml;
    // Make sure build_helpers is accessible, possibly by moving it into build_src or using a relative path.
extern crate cargo_toml_parts;
// ...
    let _generated_cargo_toml: CargoToml = define_root_cargo_toml!();
}
