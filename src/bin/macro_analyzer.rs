use anyhow::{Context, Result};
use syn::{
    parse_file,
    visit::Visit,
    Ident, LitInt, LitStr, LitBool, LitChar, LitByte, LitFloat,
    ItemFn, ExprCall, ExprMethodCall, ExprPath, Token,
};
use std::{collections::HashMap, fs, path::Path, path::PathBuf};
use walkdir;
use serde::{Serialize, Deserialize};
use toml; // Import toml crate


// Define a struct to hold extracted terms
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum Term {
    StringLiteral(String),
    NumericLiteral(String), // Store as string to handle different numeric types
    BooleanLiteral(bool),
    CharLiteral(char),
    ByteLiteral(u8),
    FloatLiteral(String), // Store as string
    Identifier(String),
    FunctionCall(String), // Function or method call name
}

// Stores the calculated scores for a term at different levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TermScores {
    local_score: f64,
    module_score: f64,
    global_score: f64,
}

// Struct to hold different levels of frequencies and scores
#[derive(Debug, Default)]
struct TermAnalysis {
    // Macro name (e.g., "dep_version_impl") -> Vec<Term> (all terms extracted from that macro)
    terms_per_macro: HashMap<String, Vec<Term>>,

    module_frequencies: HashMap<Term, usize>, // Term -> Count (for current module, i.e., macros.rs)
    global_frequencies: HashMap<Term, usize>, // Term -> Count (across all analyzed files/crates)

    total_module_terms: usize, // Total terms in current module
    total_global_terms: usize, // Total terms globally

    // Stores scores per unique term, with a local score per macro it appears in
    term_detailed_scores: HashMap<Term, (f64, f64, HashMap<String, f64>)>, // Term -> (module_score, global_score, HashMap<macro_name, local_score>)
}


#[derive(Debug, Default)]
struct TermCollector {
    terms: Vec<Term>,
}

impl<'ast> Visit<'ast> for TermCollector {
    fn visit_lit_str(&mut self, i: &'ast LitStr) {
        self.terms.push(Term::StringLiteral(i.value()));
        syn::visit::visit_lit_str(self, i);
    }

    fn visit_lit_int(&mut self, i: &'ast LitInt) {
        self.terms.push(Term::NumericLiteral(i.to_string()));
        syn::visit::visit_lit_int(self, i);
    }

    fn visit_lit_bool(&mut self, i: &'ast LitBool) {
        self.terms.push(Term::BooleanLiteral(i.value()));
        syn::visit::visit_lit_bool(self, i);
    }

    fn visit_lit_char(&mut self, i: &'ast LitChar) {
        self.terms.push(Term::CharLiteral(i.value()));
        syn::visit::visit_lit_char(self, i);
    }

    fn visit_lit_byte(&mut self, i: &'ast LitByte) {
        self.terms.push(Term::ByteLiteral(i.value()));
        syn::visit::visit_lit_byte(self, i);
    }

    fn visit_lit_float(&mut self, i: &'ast LitFloat) {
        self.terms.push(Term::FloatLiteral(i.to_string()));
        syn::visit::visit_lit_float(self, i);
    }

    fn visit_ident(&mut self, i: &'ast Ident) {
        let name = i.to_string();
        // A more conservative filter for identifiers that are likely "terms"
        // Avoid common keywords, macro internal idents, and `syn` boilerplate
        if !name.starts_with("__") && // internal syn/proc-macro2 idents
           !name.starts_with("visit_") && // visitor methods
           ![
                "self", "super", "crate", "pub", "fn", "use", "mod", "impl", "for", "in", "let",
                "mut", "if", "else", "where", "macro_rules", "async", "await", "break", "continue",
                "enum", "extern", "false", "true", "loop", "match", "return", "static", "struct",
                "trait", "type", "union", "unsafe", "while", "const", "static", "move", "dyn",
                "ref", "box", "raw", "yield", "do", "typeof", "abstract", "become", "box", "final",
                "macro", "override", "priv", "typeof", "unsized", "virtual", "try", "union",
                "static_assert", "global", "unreachable", "default", "union",
                // Common type/trait idents that are usually not "terms" unless directly referenced
                "TokenStream", "Ident", "LitStr", "Punctuated", "Package", "CargoToml", "Dependency",
                "parse_macro_input", "quote", "ToTokens", "Result", "HashMap", "Path", "fs", "std",
                "String", "u64", "bool", "char", "u8", "Clone", "PartialEq", "Eq", "Hash", "Debug",
                "Default", "Vec", "ItemFn", "ExprCall", "ExprMethodCall", "ExprPath",
                "Token", "syn", "proc_macro", "proc_macro2", "cargo_toml_generator_types", "parsers",
                "DepTableInput", "WrappedCargoToml", "serde_json", "to_string", "from_str",
                "expect", "default", "to_string", "value", "span", "first", "last", "name",
                "table_content", "members", "version", "edition", "authors", "description",
                "homepage", "include", "keywords", "license", "publish", "repository", "rust_version",
                "name_lit", "version_lit", "path_lit", "name_ident", "_comma_token", "serialized",
                "cargo_toml_json",
            ].contains(&name.as_str()) &&
           !name.ends_with("_impl") // function implementation names
        {
            self.terms.push(Term::Identifier(name));
        }
        syn::visit::visit_ident(self, i);
    }

    fn visit_expr_call(&mut self, i: &'ast ExprCall) {
        // Extract the function name from a direct call
        if let syn::Expr::Path(ExprPath { path, .. }) = &*i.func {
            if let Some(segment) = path.segments.last() {
                self.terms.push(Term::FunctionCall(segment.ident.to_string()));
            }
        }
        syn::visit::visit_expr_call(self, i);
    }

    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        // Extract the method name
        self.terms.push(Term::FunctionCall(i.method.to_string()));
        syn::visit::visit_expr_method_call(self, i);
    }
}

fn analyze_file_macros(file_path: &Path) -> Result<HashMap<String, Vec<Term>>> {
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

// Primes up to 71
const PRIMES: &[usize] = &[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71];

// Function to find the closest 1/p score
fn get_closest_prime_reciprocal(relative_frequency: f64) -> f64 {
    if relative_frequency == 0.0 {
        return 0.0;
    }
    if relative_frequency >= 1.0 { // "1 occurs all the time"
        return 1.0;
    }

    let mut closest_score = 0.0;
    let mut min_diff = f64::MAX;

    for &p in PRIMES {
        let score = 1.0 / (p as f64);
        let diff = (relative_frequency - score).abs();
        if diff < min_diff {
            min_diff = diff;
            closest_score = score;
        }
    }
    closest_score
}

// Struct for the TOML output format
#[derive(Serialize, Deserialize, Debug)]
struct MacroAnalysisOutput {
    total_module_terms: usize,
    total_global_terms: usize,
    module_frequencies: HashMap<Term, usize>,
    global_frequencies: HashMap<Term, usize>,
    term_scores_by_macro: HashMap<String, HashMap<Term, f64>>, // Macro name -> Term -> Local Score
    global_module_term_scores: HashMap<Term, (f64, f64)>, // Term -> (Module Score, Global Score)
}


fn main() -> Result<()> {
    // Directory to analyze for module terms (e.g., current crate's src)
    let module_root_dir = PathBuf::from("cargo-toml-generator-macros/src/");
    // Directories to analyze for global terms (e.g., entire project)
    let global_root_dirs = vec![
        PathBuf::from("cargo-toml-generator-macros/src/"),
        PathBuf::from("src/"), // Assuming this is part of the project's overall code
        // Add other directories for global analysis here if needed
    ];


    let mut analysis = TermAnalysis::default();
    let mut all_global_terms_vec = Vec::new(); // Collect all terms for global frequency


    // --- Module-level analysis (e.g., terms from cargo-toml-generator-macros/src/) ---
    let mut module_terms_from_files = HashMap::new(); // File path -> Vec<Term>
    for entry in walkdir::WalkDir::new(&module_root_dir)
        .context(format!("Failed to walk directory: {}", module_root_dir.display()))? {
        let entry = entry.context("Failed to read directory entry")?;
        if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "rs") {
            let file_macro_terms = analyze_file_macros(entry.path())?;
            for (macro_name, terms) in file_macro_terms {
                analysis.terms_per_macro.insert(macro_name.clone(), terms.clone());
                all_global_terms_vec.extend(terms.clone()); // Add to global pool
                module_terms_from_files.entry(entry.path().to_path_buf()).or_insert_with(Vec::new).extend(terms);
            }
        }
    }

    // Calculate module frequencies
    let mut all_module_terms_vec_for_freq = Vec::new(); // Renamed to avoid shadowing
    for (_, terms) in &module_terms_from_files {
        all_module_terms_vec_for_freq.extend(terms.clone());
    }
    analysis.total_module_terms = all_module_terms_vec_for_freq.len();
    for term in &all_module_terms_vec_for_freq {
        *analysis.module_frequencies.entry(term.clone()).or_insert(0) += 1;
    }

    // --- Global-level analysis (across all specified directories) ---
    // This is already being collected into all_global_terms_vec
    analysis.total_global_terms = all_global_terms_vec.len();
    for term in &all_global_terms_vec {
        *analysis.global_frequencies.entry(term.clone()).or_insert(0) += 1;
    }


    // Calculate scores for each unique term
    let mut term_scores_by_macro: HashMap<String, HashMap<Term, f64>> = HashMap::new();
    let mut global_module_term_scores: HashMap<Term, (f64, f64)> = HashMap::new();


    for (macro_name, terms) in &analysis.terms_per_macro {
        let mut local_counts: HashMap<Term, usize> = HashMap::new();
        for term in terms {
            *local_counts.entry(term.clone()).or_insert(0) += 1;
        }

        let total_local_terms = terms.len();
        let mut current_macro_local_scores: HashMap<Term, f64> = HashMap::new();

        for term in terms {
            let module_frequency = *analysis.module_frequencies.get(term).unwrap_or(&0);
            let global_frequency = *analysis.global_frequencies.get(term).unwrap_or(&0);
            let local_frequency = *local_counts.get(term).unwrap_or(&0);

            let module_relative_frequency = if analysis.total_module_terms > 0 {
                module_frequency as f64 / analysis.total_module_terms as f64
            } else {
                0.0
            };
            let global_relative_frequency = if analysis.total_global_terms > 0 {
                global_frequency as f64 / analysis.total_global_terms as f64
            } else {
                0.0
            };
            let local_relative_frequency = if total_local_terms > 0 {
                local_frequency as f64 / total_local_terms as f64
            } else {
                0.0
            };

            let module_score = get_closest_prime_reciprocal(module_relative_frequency);
            let global_score = get_closest_prime_reciprocal(global_relative_frequency);
            let local_score = get_closest_prime_reciprocal(local_relative_frequency);

            current_macro_local_scores.insert(term.clone(), local_score);
            global_module_term_scores.insert(term.clone(), (module_score, global_score));
        }
        term_scores_by_macro.insert(macro_name.clone(), current_macro_local_scores);
    }


    // Prepare output structure
    let output_data = MacroAnalysisOutput {
        total_module_terms: analysis.total_module_terms,
        total_global_terms: analysis.total_global_terms,
        module_frequencies: analysis.module_frequencies,
        global_frequencies: analysis.global_frequencies,
        term_scores_by_macro,
        global_module_term_scores,
    };

    // Serialize to TOML
    let toml_string = toml::to_string_pretty(&output_data)
        .context("Failed to serialize analysis results to TOML")?;

    // Create output directory if it doesn't exist
    let output_dir = PathBuf::from("output/");
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;

    // Write to file
    let output_file_path = output_dir.join("macro_scores.toml");
    fs::write(&output_file_path, toml_string)
        .with_context(|| format!("Failed to write analysis results to file: {}", output_file_path.display()))?;

    println!("Analysis results written to {}", output_file_path.display());


    // Print frequencies and scores for verification
    println!("--- Term Analysis Results ---");
    println!("\nModule Frequencies (Total terms: {})", analysis.total_module_terms);
    for (term, count) in &output_data.module_frequencies {
        println!("  {:?}: {}\n", term, count);
    }

    println!("\nGlobal Frequencies (Total terms: {})", output_data.total_global_terms);
    for (term, count) in &output_data.global_frequencies {
        println!("  {:?}: {}\n", term, count);
    }

    println!("\nTerm Scores (Module, Global, and Local per macro):");
    for (term, (module_score, global_score)) in &output_data.global_module_term_scores {
        print!("  {:?}: Module: {:.4}, Global: {:.4}", term, module_score, global_score);
        for (macro_name, local_scores_map) in &output_data.term_scores_by_macro {
            if let Some(&local_score) = local_scores_map.get(term) {
                print!(", {}: {:.4}", macro_name, local_score);
            }
        }
        println!();
    }


    Ok(())
}
