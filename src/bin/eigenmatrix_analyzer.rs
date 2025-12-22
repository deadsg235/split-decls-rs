use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

fn main() -> Result<()> {
    println!("🧮 Building Rust Ecosystem Eigenmatrix from extracted macros...");
    
    let base_path = PathBuf::from("../../");
    let mut macro_frequency: HashMap<String, usize> = HashMap::new();
    let mut macro_patterns: HashMap<String, Vec<String>> = HashMap::new();
    let mut total_macros = 0;
    
    // Scan all decls directories for macro files
    for entry in WalkDir::new(base_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        if path.is_file() && 
           path.file_name().and_then(|n| n.to_str()).map_or(false, |name| 
               name.contains("_decls_") && name.ends_with(".rs") && name.contains("macro")
           ) {
            
            if let Ok(content) = fs::read_to_string(path) {
                // Extract macro name from filename
                if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                    let parts: Vec<&str> = filename.split("_decls_").collect();
                    if parts.len() == 2 {
                        let macro_name = parts[1].to_string();
                        
                        *macro_frequency.entry(macro_name.clone()).or_insert(0) += 1;
                        macro_patterns.entry(macro_name).or_default().push(content);
                        total_macros += 1;
                    }
                }
            }
        }
    }
    
    println!("📊 Eigenmatrix Analysis Results:");
    println!("Total macros extracted: {}", total_macros);
    println!("Unique macro patterns: {}", macro_frequency.len());
    
    // Sort by frequency to find the most fundamental macros
    let mut sorted_macros: Vec<_> = macro_frequency.iter().collect();
    sorted_macros.sort_by(|a, b| b.1.cmp(a.1));
    
    println!("\n🔥 Top 20 Most Common Macros (Eigenvalues):");
    for (i, (macro_name, count)) in sorted_macros.iter().take(20).enumerate() {
        let percentage = (**count as f64 / total_macros as f64) * 100.0;
        println!("{}. {} - {} occurrences ({:.2}%)", i+1, macro_name, count, percentage);
    }
    
    // Calculate eigenmatrix dimensions
    let eigenvalue_threshold = total_macros / 100; // Top 1% by frequency
    let core_macros: Vec<_> = sorted_macros.iter()
        .filter(|(_, count)| **count >= eigenvalue_threshold)
        .map(|(name, count)| (name.to_string(), **count))
        .collect();
    
    println!("\n🎯 Core Eigenmatrix ({} fundamental macros):", core_macros.len());
    for (name, count) in &core_macros {
        println!("  {} -> {}", name, count);
    }
    
    // Save eigenmatrix data
    let eigenmatrix_data = serde_json::json!({
        "total_macros": total_macros,
        "unique_patterns": macro_frequency.len(),
        "core_macros": core_macros,
        "frequency_distribution": macro_frequency
    });
    
    fs::write("rust_eigenmatrix.json", serde_json::to_string_pretty(&eigenmatrix_data)?)?;
    println!("\n💾 Eigenmatrix saved to rust_eigenmatrix.json");
    
    Ok(())
}
