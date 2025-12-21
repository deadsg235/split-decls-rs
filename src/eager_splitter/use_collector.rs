use syn::visit::{self, Visit};
use syn::ItemUse;

/// Visitor to collect all top-level `use` statements.
#[derive(Default)]
pub struct UseStatementCollector {
    pub uses: Vec<ItemUse>,
}

impl<'ast> Visit<'ast> for UseStatementCollector {
    fn visit_item_use(&mut self, i: &'ast ItemUse) {
        self.uses.push(i.clone());
        // Call the default visitor to continue traversal for nested items if any
        // (though for top-level uses, there aren't typically nested ItemUse)
        visit::visit_item_use(self, i);
    }
}
