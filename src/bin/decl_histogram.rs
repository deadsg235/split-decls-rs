use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_path = PathBuf::from("../../");
    let mut histogram: HashMap<String, u32> = HashMap::new();
    let mut total_files = 0;
    
    println!("📊 Declaration Type Histogram Report");
    println!("=====================================");
    
    // Find all declaration files
    find_decl_files(&base_path, &mut histogram, &mut total_files)?;
    
    // Sort by count descending
    let mut sorted: Vec<_> = histogram.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    
    println!("\n📈 Declaration Types Found:");
    println!("{:<20} {:<8} {}", "Type", "Count", "Bar");
    println!("{:-<50}", "");
    
    let max_count = sorted.first().map(|(_, count)| **count).unwrap_or(0);
    
    for (decl_type, count) in sorted {
        let bar_length = if max_count > 0 { (*count * 40 / max_count).max(1) } else { 0 };
        let bar = "█".repeat(bar_length as usize);
        println!("{:<20} {:<8} {}", decl_type, count, bar);
    }
    
    println!("\n📋 Summary:");
    println!("Total declaration files: {}", total_files);
    println!("Unique declaration types: {}", histogram.len());
    
    Ok(())
}

fn find_decl_files(path: &PathBuf, histogram: &mut HashMap<String, u32>, total: &mut u32) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    let name = entry_path.file_name().unwrap().to_string_lossy();
                    if !name.starts_with('.') && name != "target" {
                        find_decl_files(&entry_path, histogram, total)?;
                    }
                } else if let Some(filename) = entry_path.file_name() {
                    let filename_str = filename.to_string_lossy();
                    if filename_str.contains("_decls_") && filename_str.ends_with(".rs") {
                        *total += 1;
                        if let Some(decl_type) = extract_decl_type(&filename_str) {
                            *histogram.entry(decl_type).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn extract_decl_type(filename: &str) -> Option<String> {
    // Extract type from patterns like: crate_decls_TypeName.rs
    if let Some(last_part) = filename.strip_suffix(".rs") {
        if let Some(type_name) = last_part.split("_decls_").nth(1) {
            // Classify by common patterns
            if type_name.starts_with("impl_for_") {
                return Some("impl".to_string());
            } else if filename.contains("_struct_") || is_likely_struct(type_name) {
                return Some("struct".to_string());
            } else if filename.contains("_enum_") || is_likely_enum(type_name) {
                return Some("enum".to_string());
            } else if filename.contains("_fn_") || is_likely_function(type_name) {
                return Some("function".to_string());
            } else if filename.contains("_const_") || type_name.chars().all(|c| c.is_uppercase() || c == '_') {
                return Some("const".to_string());
            } else if filename.contains("_trait_") {
                return Some("trait".to_string());
            } else if filename.contains("_type_") {
                return Some("type".to_string());
            } else if filename.contains("_static_") {
                return Some("static".to_string());
            } else {
                return Some("other".to_string());
            }
        }
    }
    None
}

fn is_likely_struct(name: &str) -> bool {
    name.chars().next().map_or(false, |c| c.is_uppercase())
}

fn is_likely_enum(name: &str) -> bool {
    name.chars().next().map_or(false, |c| c.is_uppercase()) && name.contains("Kind")
}

fn is_likely_function(name: &str) -> bool {
    name.chars().next().map_or(false, |c| c.is_lowercase())
}
