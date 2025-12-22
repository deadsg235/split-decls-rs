use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::macro_analyzer_parts::terms::Term;

// Struct for the TOML output format
#[derive(Serialize, Deserialize, Debug)]
pub struct MacroAnalysisOutput {
    pub total_module_terms: usize,
    pub total_global_terms: usize,
    pub module_frequencies: HashMap<Term, usize>,
    pub global_frequencies: HashMap<Term, usize>,
    pub term_scores_by_macro: HashMap<String, HashMap<Term, f64>>, // Macro name -> Term -> Local Score
    pub global_module_term_scores: HashMap<Term, (f64, f64)>, // Term -> (Module Score, Global Score)
}
