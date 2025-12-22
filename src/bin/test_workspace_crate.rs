use split_decls_rs::{setup_crate_paths, eager_splitter};
use split_decls_types::SplitDeclsConfig;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    println!("Testing direct eager splitter on a workspace crate...");
    
    // Test on a simple crate from the workspace
    let crate_path = PathBuf::from("../../crates/monster_traits");
    
    // Check if the crate exists and has a lib.rs
    if !crate_path.join("src/lib.rs").exists() {
        println!("Crate doesn't have src/lib.rs, skipping...");
        return Ok(());
    }
    
    println!("Processing crate: {}", crate_path.display());
    
    let paths = setup_crate_paths(&crate_path)?;
    let config = SplitDeclsConfig::default();
    
    println!("Output will be generated to: {}", paths.decls_output_dir.display());
    
    // Run the eager splitter
    eager_splitter::eager_split_crate(&paths, &config)?;
    
    println!("✅ Successfully processed crate!");
    println!("Check the output directory: {}", paths.decls_output_dir.display());
    
    Ok(())
}
