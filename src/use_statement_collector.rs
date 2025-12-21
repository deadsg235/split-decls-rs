/// Visitor to collect all top-level `use` statements.
#[derive(Default)]
pub struct UseStatementCollector {
    pub uses: Vec<ItemUse>,
}

impl<'ast> Visit<'ast> for UseStatementCollector {
    fn visit_item_use(&mut self, i: &'ast ItemUse) {
        self.uses.push(i.clone());
        visit::visit_item_use(self, i); // Corrected recursive call
    }
}
