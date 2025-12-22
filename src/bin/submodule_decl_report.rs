use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn main() -> anyhow::Result<()> {
    let base_path = Path::new("../../");
    let mut submodule_stats: HashMap<String, HashMap<String, usize>> = HashMap::new();
    
    for entry in WalkDir::new(base_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        if path.file_name().and_then(|n| n.to_str()) == Some("decls") && path.is_dir() {
            if let Some(submodule_name) = extract_submodule_name(path) {
                let mut type_counts: HashMap<String, usize> = HashMap::new();
                
                for decl_file in fs::read_dir(path)? {
                    let decl_file = decl_file?;
                    if decl_file.path().extension().and_then(|s| s.to_str()) == Some("rs") {
                        if let Some(decl_type) = extract_decl_type(&decl_file.path()) {
                            *type_counts.entry(decl_type).or_insert(0) += 1;
                        }
                    }
                }
                
                if !type_counts.is_empty() {
                    submodule_stats.insert(submodule_name, type_counts);
                }
            }
        }
    }
    
    println!("Declaration Report by Submodule:");
    println!("================================");
    
    for (submodule, types) in &submodule_stats {
        let total: usize = types.values().sum();
        println!("\n{} (total: {})", submodule, total);
        
        let mut sorted_types: Vec<_> = types.iter().collect();
        sorted_types.sort_by(|a, b| b.1.cmp(a.1));
        
        for (decl_type, count) in sorted_types {
            println!("  {}: {}", decl_type, count);
        }
    }
    
    Ok(())
}

fn extract_submodule_name(decls_path: &Path) -> Option<String> {
    decls_path.parent()?.parent()?.file_name()?.to_str().map(|s| s.to_string())
}

fn extract_decl_type(file_path: &Path) -> Option<String> {
    let filename = file_path.file_stem()?.to_str()?;
    
    if filename.contains("_struct_") { Some("struct".to_string()) }
    else if filename.contains("_enum_") { Some("enum".to_string()) }
    else if filename.contains("_impl_") { Some("impl".to_string()) }
    else if filename.contains("_fn_") { Some("function".to_string()) }
    else if filename.contains("_const_") { Some("const".to_string()) }
    else if filename.contains("_trait_") { Some("trait".to_string()) }
    else if filename.contains("_type_") { Some("type".to_string()) }
    else if filename.contains("_static_") { Some("static".to_string()) }
    else { Some("other".to_string()) }
}
