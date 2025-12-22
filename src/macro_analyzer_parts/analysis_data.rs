use std::collections::HashMap;
use crate::macro_analyzer_parts::terms::Term; // Assuming Term is in terms.rs

#[derive(Debug, Default)]
pub struct TermAnalysis {
    // Macro name (e.g., "dep_version_impl") -> Vec<Term> (all terms extracted from that macro)
    pub terms_per_macro: HashMap<String, Vec<Term>>,

    pub module_frequencies: HashMap<Term, usize>, // Term -> Count (for current module, i.e., macros.rs)
    pub global_frequencies: HashMap<Term, usize>, // Term -> Count (across all analyzed files/crates)

    pub total_module_terms: usize, // Total terms in current module
    pub total_global_terms: usize, // Total terms globally

    // Stores scores per unique term, with a local score per macro it appears in
    pub term_detailed_scores: HashMap<Term, (f64, f64, HashMap<String, f64>)>, // Term -> (module_score, global_score, HashMap<macro_name, local_score>)
}
