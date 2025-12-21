use proc_macro2::TokenStream;
/// Represents a single extracted declaration.
pub struct ExtractedDecl {
    pub name: String,
    pub kind: String, // e.g., "fn", "struct", "enum"
    pub content: TokenStream,
}
