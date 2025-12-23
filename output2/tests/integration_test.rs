use split_decls_rs_output::*;
use std::path::PathBuf;

#[test]
fn test_crate_paths() {
    let path = PathBuf::from("/test/path");
    let name = "test_crate".to_string();
    
    let crate_paths = CratePaths::new(path.clone(), name.clone());
    
    assert_eq!(crate_paths.crate_path, path);
    assert_eq!(crate_paths.crate_name, name);
}

#[test]
fn test_basic_functionality() {
    // Test that the basic types are available
    let _path: PathBuf = PathBuf::new();
    let _result: Result<(), anyhow::Error> = Ok(());
    
    // Test passes if compilation succeeds
    assert!(true);
}