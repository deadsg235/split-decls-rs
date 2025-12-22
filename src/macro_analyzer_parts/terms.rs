use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt;

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

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Term::StringLiteral(s) => write!(f, "{}", s),
            Term::NumericLiteral(s) => write!(f, "{}", s),
            Term::BooleanLiteral(b) => write!(f, "{}", b),
            Term::CharLiteral(c) => write!(f, "{}", c),
            Term::ByteLiteral(b) => write!(f, "{}", b),
            Term::FloatLiteral(s) => write!(f, "{}", s),
            Term::Identifier(s) => write!(f, "{}", s),
            Term::FunctionCall(s) => write!(f, "{}", s),
        }
    }
}

// Stores the calculated scores for a term at different levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TermScores {
    pub local_score: f64,
    pub module_score: f64,
    pub global_score: f64,
}
