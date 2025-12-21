// build_script_modules/example_module.rs
// This module contains example build logic that will be composed into the main build.rs.

pub fn run() {
    println!("cargo:warning=Running build logic from example_module!");
    // Add your specific build logic here.
    // For instance, you could generate a file, print some information, etc.
}
