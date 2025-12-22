use split_decls_rs::setup_crate_paths;
use std::path::PathBuf;

fn main() {
    println!("Testing CratePaths generation with output directory...");
    
    let crate_path = PathBuf::from("test_crate");
    
    let paths = setup_crate_paths(&crate_path).unwrap();
    
    println!("Crate name: {}", paths.crate_name);
    println!("Crate path: {}", paths.crate_path.display());
    println!("Lib.rs path: {}", paths.lib_rs_path.display());
    println!("Old lib.rs path: {}", paths.old_lib_rs_path.display());
    println!("Decls output dir: {}", paths.decls_output_dir.display());
    
    // Verify the output directory structure
    let expected_output = PathBuf::from("output").join("test_crate").join("src").join("decls");
    assert_eq!(paths.decls_output_dir, expected_output);
    
    println!("✅ Output directory path is correct: {}", expected_output.display());
    println!("✅ Test passed!");
}
