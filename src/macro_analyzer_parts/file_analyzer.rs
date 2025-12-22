use anyhow::{Context, Result};
use syn::visit::Visit;
use syn::{
    parse_file,
};
use std::{collections::HashMap, fs, path::Path};
use crate::macro_analyzer_parts::terms::Term;
use crate::macro_analyzer_parts::term_collector::TermCollector;

pub fn analyze_file_macros(file_path: &Path) -> Result<HashMap<String, Vec<Term>>> {
    let mut content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    // Pre-processing steps to clean up malformed macro output
    // 1. Remove ". sig" suffix if present
    if content.trim_end().ends_with(". sig") {
        let trimmed_len = content.trim_end().len();
        content.truncate(trimmed_len - ". sig".len());
        // Now, we *don't* return an empty HashMap; we attempt to parse the cleaned content.
    }

    // 2. Replace "# [" with "#["
    content = content.replace("# [", "#[");

    // 3. Add newlines after closing braces and brackets for better parsing chances (heuristic)
    content = content.replace(" } ", "}\n");
    content = content.replace(" ] ", "]\n");
    let ast = parse_file(&content)
        .with_context(|| format!("Failed to parse Rust file: {}", file_path.display()))?;

    let mut macro_terms: HashMap<String, Vec<Term>> = HashMap::new();

    for item in ast.items {
        if let syn::Item::Fn(func) = item {
            // Check for explicit `pub` keyword in visibility
            let is_pub = matches!(func.vis, syn::Visibility::Public(_));
            if func.sig.ident.to_string().ends_with("_impl") && is_pub {
                let mut collector = TermCollector::default();
                collector.visit_item_fn(&func);
                macro_terms.insert(func.sig.ident.to_string(), collector.terms);
            }
        }
    }
    Ok(macro_terms)
}
