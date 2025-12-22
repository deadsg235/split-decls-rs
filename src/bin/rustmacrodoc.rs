use anyhow::{Context, Result};
use clap::Parser;
use quote; // Added quote for quote::quote!
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Attribute, ItemFn, ItemMacro, Lit, LitStr, Macro};
use toml;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct MacroAnalysis {
    uses_syn: bool,
    defines_const: bool,
    defines_enum: bool,
    defines_struct: bool,
    calls_other_macros: Vec<String>,
}

/// Represents information about a single macro.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct MacroInfo {
    name: String,
    kind: String, // e.g., "macro_rules", "proc_macro", "proc_macro_attribute", "proc_macro_derive"
    file: String,
    // Note: `proc_macro2::Span::start().line/column` is unstable.
    // Storing debug string of the span as a workaround.
    span_debug_string: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<String>, // For proc macros, or `macro_rules! name { ... }`
    #[serde(skip_serializing_if = "Option::is_none")]
    doc_comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    analysis: Option<MacroAnalysis>,
}

#[derive(Debug, Serialize, Deserialize)]
enum FileStatus {
    Ok,
    ParsingError,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileMetadata {
    file_path: String,
    content_hash: String,
    status: FileStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    macros: Option<Vec<MacroInfo>>, // Only present if status is Ok
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>, // Only present if status is ParsingError
    // Future fields for parse tree summary, compiler data etc.
}

/// The report structure containing all discovered macros.
#[derive(Debug, Serialize, Deserialize)]
struct MacroReport {
    files: Vec<FileMetadata>,
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
            span_debug_string: format!("{:?}", i.mac.span()),
            signature: Some(quote::quote! { #i }.to_string()), // Capture the whole macro invocation for signature
            doc_comment: get_doc_comment(&i.attrs),
            analysis: None,
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
                span_debug_string: format!("{:?}", i.span()),
                signature: Some(quote::quote! { #i }.to_string()), // Capture the full function item
                doc_comment: get_doc_comment(&i.attrs),
                analysis: None,
            });
        }
        syn::visit::visit_item_fn(self, i);
    }
}

// Helper to extract doc comments
fn get_doc_comment(attrs: &[Attribute]) -> Option<String> {
    let mut doc_comments = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(nv) = &attr.meta {
                                if let syn::Expr::Lit(expr_lit) = &nv.value {
                                    if let Lit::Str(lit_str) = &expr_lit.lit {
                                        doc_comments.push(lit_str.value().trim().to_string());
                                    }
                                }            }
        }
    }
    if doc_comments.is_empty() {
        None
    } else {
        Some(doc_comments.join("\n"))
    }
}

struct IncludeVisitor {
    includes: Vec<PathBuf>,
    current_file_path: PathBuf,
}

impl<'ast> Visit<'ast> for IncludeVisitor {
    fn visit_macro(&mut self, mac: &'ast Macro) {
        if mac.path.is_ident("include") {
            if let Ok(lit_str) = mac.parse_body::<LitStr>() {
                let include_path = self
                    .current_file_path
                    .parent()
                    .unwrap()
                    .join(lit_str.value());
                self.includes.push(include_path);
            }
        }
        syn::visit::visit_macro(self, mac);
    }
}

struct CategorizationVisitor {
    analysis: MacroAnalysis,
}

impl<'ast> Visit<'ast> for CategorizationVisitor {
    fn visit_item_use(&mut self, i: &'ast syn::ItemUse) {
        if format!("{}", quote::quote!(#i)).contains("syn") {
            self.analysis.uses_syn = true;
        }
        syn::visit::visit_item_use(self, i);
    }

    fn visit_item_const(&mut self, i: &'ast syn::ItemConst) {
        self.analysis.defines_const = true;
        syn::visit::visit_item_const(self, i);
    }

    fn visit_item_enum(&mut self, i: &'ast syn::ItemEnum) {
        self.analysis.defines_enum = true;
        syn::visit::visit_item_enum(self, i);
    }

    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        self.analysis.defines_struct = true;
        syn::visit::visit_item_struct(self, i);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if let Some(segment) = mac.path.segments.last() {
            self.analysis.calls_other_macros.push(segment.ident.to_string());
        }
        syn::visit::visit_macro(self, mac);
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Parser)]
enum Commands {
    /// Scan a directory for macros and produce a report
    Scan {
        /// Path to scan
        #[arg()]
        path: Option<PathBuf>,

        /// Scan all local dependencies in the workspace
        #[arg(short, long)]
        workspace: bool,

        /// Output file for the report
        #[arg(short, long, default_value = "report.json")]
        output: PathBuf,
    },
    /// Split macros from a report file into separate files
    Split {
        /// Path to the report.json file
        #[arg()]
        report_path: PathBuf,

        /// Output directory for macros
        #[arg(short, long, default_value = "output/macros")]
        output: PathBuf,
    },
}

struct StringLiteralVisitor {
    strings: Vec<String>,
}

impl<'ast> Visit<'ast> for StringLiteralVisitor {
    fn visit_lit_str(&mut self, lit_str: &'ast LitStr) {
        self.strings.push(lit_str.value());
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan {
            path,
            workspace,
            output,
        } => {
            let mut paths_to_scan: Vec<PathBuf> = Vec::new();
            let mut scanned_paths: HashSet<PathBuf> = HashSet::new();

            if workspace {
                println!("Scanning workspace...");

                // Find workspace root
                let mut current_dir = std::env::current_dir()?;
                let mut workspace_root = None;
                loop {
                    let cargo_toml_path = current_dir.join("Cargo.toml");
                    if cargo_toml_path.exists() {
                        let content = fs::read_to_string(&cargo_toml_path)?;
                        let toml_val: toml::Value = toml::from_str(&content)?;
                        if toml_val.get("workspace").is_some() {
                            workspace_root = Some(current_dir);
                            break;
                        }
                    }
                    if !current_dir.pop() {
                        break;
                    }
                }

                if let Some(root) = workspace_root {
                    println!("Found workspace root at: {}", root.display());
                    let cargo_toml: toml::Value = toml::from_str(&fs::read_to_string(root.join("Cargo.toml"))?)?;

                    if let Some(workspace_table) = cargo_toml.get("workspace").and_then(|w| w.as_table()) {
                        if let Some(members) = workspace_table.get("members").and_then(|m| m.as_array()) {
                            for member in members {
                                if let Some(member_str) = member.as_str() {
                                    let member_path = root.join(member_str);
                                    if member_path.exists() {
                                        paths_to_scan.push(member_path);
                                    } else {
                                        println!("Warning: workspace member path does not exist: {}", member_path.display());
                                    }
                                }
                            }
                        }
                    }
                } else {
                    println!("Warning: Could not find workspace root. Scanning current directory only.");
                    paths_to_scan.push(PathBuf::from("."));
                }

            } else if let Some(path) = path {
                paths_to_scan.push(path);
            } else {
                paths_to_scan.push(PathBuf::from("."));
            }

            let mut all_files_metadata: Vec<FileMetadata> = Vec::new();

            while let Some(path_to_scan) = paths_to_scan.pop() {
                let canonical_path_to_scan = match path_to_scan.canonicalize() {
                    Ok(p) => p,
                    Err(_) => {
                        println!("Warning: could not canonicalize path: {}", path_to_scan.display());
                        continue;
                    }
                };

                if !scanned_paths.insert(canonical_path_to_scan.clone()) {
                    continue;
                }

                println!("Scanning: {}", path_to_scan.display());

                for entry in WalkDir::new(&path_to_scan).into_iter().filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                        let canonical_path = match path.canonicalize() {
                            Ok(p) => p,
                            Err(_) => continue, // Ignore paths we can't canonicalize
                        };
                        if !scanned_paths.insert(canonical_path) {
                            continue;
                        }

                        let file_content_bytes = match fs::read(path) {
                            Ok(bytes) => bytes,
                            Err(_) => continue, // Ignore files we can't read
                        };

                        let content_hash = format!("{:x}", Sha256::digest(&file_content_bytes));
                        let file_content = match String::from_utf8(file_content_bytes) {
                            Ok(content) => content,
                            Err(_) => continue, // Ignore non-utf8 files
                        };

                        let mut file_metadata = FileMetadata {
                            file_path: path.display().to_string(),
                            content_hash,
                            status: FileStatus::Ok,
                            macros: None,
                            error: None,
                        };

                        let syntax_tree = match syn::parse_file(&file_content) {
                            Ok(tree) => tree,
                            Err(err) => {
                                file_metadata.status = FileStatus::ParsingError;
                                file_metadata.error = Some(err.to_string());
                                all_files_metadata.push(file_metadata);
                                continue;
                            }
                        };

                        let mut macro_visitor = MacroVisitor {
                            macros: Vec::new(),
                            file_path: path.to_path_buf(),
                        };
                        macro_visitor.visit_file(&syntax_tree);

                        let mut updated_macros = Vec::new();
                        for mut macro_info in macro_visitor.macros {
                            if let Some(signature) = &macro_info.signature {
                                if let Ok(item) = syn::parse_str::<syn::Item>(signature) {
                                    let mut categorization_visitor = CategorizationVisitor {
                                        analysis: MacroAnalysis::default(),
                                    };
                                    categorization_visitor.visit_item(&item);
                                    macro_info.analysis = Some(categorization_visitor.analysis);

                                    // Recursive scanning from string literals
                                    let mut string_visitor = StringLiteralVisitor { strings: Vec::new() };
                                    string_visitor.visit_item(&item);
            
                                    for s in string_visitor.strings {
                                        if s.contains('/') {
                                            let new_path = PathBuf::from(s);
                                            if new_path.exists() {
                                                paths_to_scan.push(new_path);
                                            }
                                        }
                                    }
                                }
                            }
                            updated_macros.push(macro_info);
                        }
                        
                        file_metadata.macros = Some(updated_macros);

                        let mut include_visitor = IncludeVisitor {
                            includes: Vec::new(),
                            current_file_path: path.to_path_buf(),
                        };
                        include_visitor.visit_file(&syntax_tree);
                        paths_to_scan.extend(include_visitor.includes);

                        all_files_metadata.push(file_metadata);
                    }
                }
            }

            let report = MacroReport {
                files: all_files_metadata,
            };

            let json_report = serde_json::to_string_pretty(&report)
                .context("Failed to serialize macro report to JSON")?;
            
            fs::write(&output, json_report)?;
            println!("Successfully generated report at {}", output.display());
        }
        Commands::Split {
            report_path,
            output,
        } => {
            let report_content = fs::read_to_string(&report_path)
                .with_context(|| format!("Failed to read report file: {}", report_path.display()))?;
            let report: MacroReport = serde_json::from_str(&report_content)
                .context("Failed to deserialize report JSON")?;

            fs::create_dir_all(&output)
                .with_context(|| format!("Failed to create output directory: {}", output.display()))?;

            let mut macros_by_name: HashMap<String, Vec<MacroInfo>> = HashMap::new();
            for file_metadata in &report.files {
                if let Some(macros) = &file_metadata.macros {
                    for macro_info in macros {
                        macros_by_name
                            .entry(macro_info.name.clone())
                            .or_default()
                            .push(macro_info.clone());
                    }
                }
            }

            let mut total_macros_split = 0;
            for (name, macros) in macros_by_name {
                if macros.is_empty() || name.is_empty() {
                    continue;
                }
                if macros.len() == 1 {
                    if let Some(signature) = macros[0].signature.clone() {

                        let file_name = format!("{}.rs", name);
                        let output_path = output.join(&file_name);
                        fs::write(&output_path, signature).with_context(|| {
                            format!("Failed to write macro to file: {}", output_path.display())
                        })?;
                        total_macros_split += 1;
                    }
                } else {
                    for macro_info in macros {
                        if let Some(signature) = macro_info.signature.clone() {

                            let mut hasher = Sha256::new();
                            hasher.update(signature.as_bytes());
                            let hash_result = hasher.finalize();
                            let short_hash = &format!("{:x}", hash_result)[..7];

                            let source_module = Path::new(&macro_info.file)
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .replace("-", "_");

                            let file_name = format!("{}-{}-{}.rs", name, source_module, short_hash);
                            let output_path = output.join(&file_name);
                            fs::write(&output_path, signature).with_context(|| {
                                format!("Failed to write macro to file: {}", output_path.display())
                            })?;
                            total_macros_split += 1;
                        }
                    }
                }
            }

            println!(
                "Successfully split {} macros into {}.",
                total_macros_split,
                output.display()
            );
        }
    }

    Ok(())
}
