use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// Define a struct to hold extracted terms
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Term {
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
pub struct TermScores {
    pub local_score: f64,
    pub module_score: f64,
    pub global_score: f64,
}
