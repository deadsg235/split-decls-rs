use proc_macro2::TokenStream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap; // Needed for future metadata fields

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct ExtractedDeclMetadata {
    pub ast_depth: usize,
    pub ast_node_count: usize, // A simple "weight"
    pub inputs_hash: Option<String>, // Hash of identifiers it depends on
    pub output_hash: Option<String>, // Hash of its own content (hash of decl.content)
    pub rings_of_sizes: Vec<usize>, // Placeholder for rings of sizes
    // Placeholder for more complex topology, partitioning, clustering info
    pub zkp_witness_hash: Option<String>, // Overall hash of this metadata
}

/// Represents a single extracted declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtractedDecl {
    pub name: String,
    pub kind: String, // e.g., "fn", "struct", "enum"
    pub content: TokenStream,
    pub metadata: ExtractedDeclMetadata, // New field
}