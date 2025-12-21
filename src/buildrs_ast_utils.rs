use proc_macro2::TokenStream;
use syn::{
    //visit::{self, Visit},
    ItemUse,
};
use syn::visit::{self, Visit};
/// Represents a single extracted declaration.
pub struct ExtractedDecl {
    pub name: String,
    pub kind: String, // e.g., "fn", "struct", "enum"
    pub content: TokenStream,
}

/// Visitor to collect all top-level `use` statements.
#[derive(Default)]
pub struct UseStatementCollector {
    pub uses: Vec<ItemUse>,
}

impl<'ast> Visit<'ast> for UseStatementCollector {
    fn visit_item_use(&mut self, i: &'ast ItemUse) {
        self.uses.push(i.clone());
        self.visit_item_use(i);
    }
}
