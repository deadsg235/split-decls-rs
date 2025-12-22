use anyhow::Context;
use std::{collections::HashMap, fs, path::Path, path::PathBuf};
use walkdir;
use toml;


use split_decls_rs::macro_analyzer_parts::terms::{Term, TermScores};
use split_decls_rs::macro_analyzer_parts::analysis_data::TermAnalysis;
use split_decls_rs::macro_analyzer_parts::term_collector::TermCollector;
use split_decls_rs::macro_analyzer_parts::file_analyzer::analyze_file_macros;
use split_decls_rs::macro_analyzer_parts::scoring::get_closest_prime_reciprocal;
use split_decls_rs::macro_analyzer_parts::output_format::MacroAnalysisOutput;
use split_decls_rs::special_print::specialprint;


fn main() -> anyhow::Result<()> {
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
    for entry in walkdir::WalkDir::new(&module_root_dir) {
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

    // Call the new function to format scores and create output_data
    let output_data = calculate_and_format_scores(
        analysis,
        term_scores_by_macro,
        global_module_term_scores,
    );

    // Serialize to TOML
    let toml_string = toml::to_string_pretty(&output_data)
        .context("Failed to serialize analysis results to TOML")?;

    // Create output directory if it doesn’t exist
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
    println!("\nModule Frequencies (Total terms: {})", output_data.total_module_terms);
    for (term, count) in &output_data.module_frequencies {
        println!("  {:?}: {}\n", term, count);
    }

    println!("\nGlobal Frequencies (Total terms: {})", output_data.total_global_terms);
    let mut sorted_global_frequencies: Vec<(&String, &usize)> = output_data.global_frequencies.iter().collect();
    sorted_global_frequencies.sort_by(|a, b| b.1.cmp(a.1));
    for (term, count) in sorted_global_frequencies {
        println!("  {:?}: {}\n", term, count);
    }

    println!("\nTerm Scores (Module, Global, and Local per macro):");
    for (term, (module_score, global_score)) in &output_data.global_module_term_scores {
        print!("  {:?}: Module: {:.4}, Global: {:.4}", term, module_score, global_score);
        for (macro_name, local_scores_map) in &output_data.term_scores_by_macro {
            if let Some(&local_score) = local_scores_map.get(term) {
                specialprint(macro_name, local_score);
            }
        }
        println!();
    }


    Ok(())
}


    


    fn calculate_and_format_scores(


        analysis: TermAnalysis,


        term_scores_by_macro: HashMap<String, HashMap<Term, f64>>,


        global_module_term_scores: HashMap<Term, (f64, f64)>,


    ) -> MacroAnalysisOutput {


        // Prepare output structure by converting Term keys to String keys for TOML serialization


        let mut output_module_frequencies = HashMap::new();


        for (term, count) in analysis.module_frequencies {


            output_module_frequencies.insert(term.to_string(), count);


        }


    


        let mut output_global_frequencies = HashMap::new();


        for (term, count) in analysis.global_frequencies {


            output_global_frequencies.insert(term.to_string(), count);


        }


    


        let mut output_term_scores_by_macro = HashMap::new();


        for (macro_name, term_map) in term_scores_by_macro {


            let mut inner_map = HashMap::new();


            for (term, score) in term_map {


                inner_map.insert(term.to_string(), score);


            }


            output_term_scores_by_macro.insert(macro_name, inner_map);


        }


    


        let mut output_global_module_term_scores = HashMap::new();


        for (term, scores) in global_module_term_scores {


            output_global_module_term_scores.insert(term.to_string(), scores);


        }


    


        MacroAnalysisOutput {


            total_module_terms: analysis.total_module_terms,


            total_global_terms: analysis.total_global_terms,


            module_frequencies: output_module_frequencies,


            global_frequencies: output_global_frequencies,


            term_scores_by_macro: output_term_scores_by_macro,


            global_module_term_scores: output_global_module_term_scores,


        }


    }


    
