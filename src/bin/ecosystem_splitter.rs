use anyhow::Result;
use split_decls_rs::process_crate;
use split_decls_types::SplitDeclsConfig;
use std::path::PathBuf;
use std::fs;
use std::time::Instant;
use std::collections::HashMap;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let base_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("../../")
    };
    
    let recursive = args.contains(&"--recursive".to_string());
    
    println!("🚀 Starting ecosystem-wide declaration splitting");
    println!("📂 Base path: {}", base_path.display());
    println!("🔄 Recursive: {}", recursive);
    
    let start_time = Instant::now();
    
    let config = SplitDeclsConfig {
        active_overlay_modules: Some(vec![]),
        custom_prelude_overlay: Some("// Split declarations prelude\nuse proc_macro::TokenStream;\nuse quote::quote;\nuse syn::*;".to_string()),
        rustc_source_path: None,
        patches: Some(std::collections::HashMap::new()),
        string_replacements: None,
        crates_io_patches: Some(std::collections::HashMap::new()),
        github_org: None,
        default_branches_to_patch: vec![],
        repo_fork_mapping: std::collections::HashMap::new(),
        workspace_dependencies: std::collections::HashMap::new(),
    };
    
    let mut processed = 0;
    let mut errors = 0;
    
    if recursive {
        // First, count total crates for progress tracking
        println!("🔍 Counting total crates...");
        let total_crates = count_crates(&base_path)?;
        println!("📊 Found {} crates to process", total_crates);
        
        process_recursive(&base_path, &config, &mut processed, &mut errors, &start_time, Some(total_crates))?;
    } else {
        if let Err(e) = process_crate(&base_path, &config, false) {
            println!("❌ Error processing {}: {}", base_path.display(), e);
            errors += 1;
        } else {
            processed += 1;
        }
    }
    
    let total_time = start_time.elapsed().as_secs_f64();
    println!("✅ Ecosystem transformation complete!");
    println!("📊 Processed: {} crates in {:.1}m", processed, total_time / 60.0);
    println!("❌ Errors: {} crates", errors);
    println!("⚡ Average rate: {:.1} crates/min", processed as f64 / (total_time / 60.0));
    
    Ok(())
}

fn count_crates(path: &PathBuf) -> Result<usize> {
    let mut count = 0;
    let mut dirs_scanned = 0;
    
    println!("🔍 Scanning: {}", path.display());
    
    if path.join("Cargo.toml").exists() && path.join("src/lib.rs").exists() {
        count += 1;
        println!("📦 Found crate: {}", path.display());
    }
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                if entry_path.is_dir() && !entry_path.is_symlink() {
                    let name = entry_path.file_name().unwrap().to_string_lossy();
                    if !name.starts_with('.') && name != "target" && name != "node_modules" {
                        dirs_scanned += 1;
                        println!("📁 Entering: {}", entry_path.display());
                        count += count_crates(&entry_path)?;
                    }
                }
            }
        }
    }
    
    Ok(count)
}

fn process_recursive(path: &PathBuf, config: &SplitDeclsConfig, processed: &mut i32, errors: &mut i32, start_time: &Instant, total_crates: Option<usize>) -> Result<()> {
    // Look for Cargo.toml files with src/lib.rs
    if path.join("Cargo.toml").exists() && path.join("src/lib.rs").exists() {
        let crate_start = Instant::now();
        println!("🔧 Processing: {}", path.display());
        
        match process_crate(path, config, false) {
            Ok(_) => {
                *processed += 1;
                let elapsed = start_time.elapsed().as_secs_f64();
                let crate_time = crate_start.elapsed().as_secs_f64();
                
                if let Some(total) = total_crates {
                    let progress = (*processed as f64 / total as f64) * 100.0;
                    let rate = *processed as f64 / elapsed;
                    let remaining = total - *processed as usize;
                    let eta_seconds = remaining as f64 / rate;
                    
                    println!("✅ Completed {} ({:.1}% done, {:.1}s, ETA: {:.0}m {:.0}s)", 
                        path.display(), progress, crate_time, eta_seconds / 60.0, eta_seconds % 60.0);
                } else {
                    println!("✅ Completed {} ({:.1}s, rate: {:.1}/min)", 
                        path.display(), crate_time, (*processed as f64 / elapsed) * 60.0);
                }
                
                if *processed % 50 == 0 {
                    println!("📈 Milestone: {} crates processed in {:.1}m", processed, elapsed / 60.0);
                }
            }
            Err(e) => {
                println!("❌ Error in {}: {}", path.display(), e);
                *errors += 1;
            }
        }
    }
    
    // Recurse into subdirectories
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                if entry_path.is_dir() && !entry_path.is_symlink() {
                    let name = entry_path.file_name().unwrap().to_string_lossy();
                    // Skip common non-source directories
                    if !name.starts_with('.') && name != "target" && name != "node_modules" {
                        process_recursive(&entry_path, config, processed, errors, start_time, total_crates)?;
                    }
                }
            }
        }
    }
    
    Ok(())
}
