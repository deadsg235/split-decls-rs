use anyhow::{Context, Result};
use syn::visit::Visit;
use syn::{
    parse_file,
    ItemFn, Token,
};
use std::{collections::HashMap, fs, path::Path};
use crate::macro_analyzer_parts::terms::Term;
use crate::macro_analyzer_parts::term_collector::TermCollector;

pub fn analyze_file_macros(file_path: &Path) -> Result<HashMap<String, Vec<Term>>> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
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
