use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::Visit;
use syn::{Attribute, Ident, Item, ItemFn, Lit, Macro, PathSegment};
use walkdir::WalkDir;

/// Represents information about a single macro.
#[derive(Debug, Serialize, Deserialize)]
struct MacroInfo {
    name: String,
    kind: String, // e.g., "macro_rules", "proc_macro", "proc_macro_attribute", "proc_macro_derive"
    file: String,
    line: usize,
    column: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<String>, // For proc macros, or `macro_rules! name { ... }`
    #[serde(skip_serializing_if = "Option::is_none")]
    doc_comment: Option<String>,
}

/// The report structure containing all discovered macros.
#[derive(Debug, Serialize, Deserialize)]
struct MacroReport {
    macros: Vec<MacroInfo>,
}

struct MacroVisitor {
    macros: Vec<MacroInfo>,
    file_path: PathBuf,
}

impl<'ast> Visit<'ast> for MacroVisitor {
    fn visit_item_macro(&mut self, i: &'ast ItemMacro) {
        // Handle macro_rules!
        self.macros.push(MacroInfo {
            name: i.mac.path.segments.last().map_or("".to_string(), |s| s.ident.to_string()),
            kind: "macro_rules".to_string(),
            file: self.file_path.display().to_string(),
            line: i.mac.span().start().line,
            column: i.mac.span().start().column,
            signature: Some(quote::quote! { #i }.to_string()), // Capture the whole macro invocation for signature
            doc_comment: get_doc_comment(&i.attrs),
        });
        syn::visit::visit_item_macro(self, i);
    }

    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        // Handle procedural macros
        let mut is_proc_macro = false;
        let mut macro_kind = String::new();

        for attr in &i.attrs {
            if attr.path().is_ident("proc_macro") {
                is_proc_macro = true;
                macro_kind = "proc_macro".to_string();
                break;
            } else if attr.path().is_ident("proc_macro_attribute") {
                is_proc_macro = true;
                macro_kind = "proc_macro_attribute".to_string();
                break;
            } else if attr.path().is_ident("proc_macro_derive") {
                is_proc_macro = true;
                macro_kind = "proc_macro_derive".to_string();
                break;
            }
        }

        if is_proc_macro {
            self.macros.push(MacroInfo {
                name: i.sig.ident.to_string(),
                kind: macro_kind,
                file: self.file_path.display().to_string(),
                line: i.span().start().line,
                column: i.span().start().column,
                signature: Some(quote::quote! { #i.sig }.to_string()), // Capture the function signature
                doc_comment: get_doc_comment(&i.attrs),
            });
        }
        syn::visit::visit_item_fn(self, i);
    }
}

// Helper to extract doc comments
// Note: This needs `syn::Meta` to be in scope.
fn get_doc_comment(attrs: &[Attribute]) -> Option<String> {
    let mut doc_comments = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(nv) = &attr.meta {
                if let Lit::Str(lit_str) = &nv.value {
                    doc_comments.push(lit_str.value().trim().to_string());
                }
            }
        }
    }
    if doc_comments.is_empty() {
        None
    } else {
        Some(doc_comments.join("\n"))
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let root_path_str = args
        .get(1)
        .map_or(".", |s| s.as_str()); // Default to current directory

    let root_path = PathBuf::from(root_path_str);
    if !root_path.exists() {
        anyhow::bail!("Root path does not exist: {}", root_path.display());
    }

    let mut all_macros_info = Vec::new();

    for entry in WalkDir::new(&root_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
            let file_content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;

            let syntax_tree = syn::parse_file(&file_content)
                .with_context(|| format!("Failed to parse file: {}", path.display()))?;

            let mut visitor = MacroVisitor {
                macros: Vec::new(),
                file_path: path.to_path_buf(),
            };
            visitor.visit_file(&syntax_tree);
            all_macros_info.extend(visitor.macros);
        }
    }

    let report = MacroReport {
        macros: all_macros_info,
    };

    let toml_report = toml::to_string_pretty(&report)
        .context("Failed to serialize macro report to TOML")?;

    println!("{}", toml_report);

    Ok(())
}