make_mut_visitor_impl ! { InvocationCollector , fn flat_map_item (& mut self , item : Box < ast :: Item >) -> SmallVec < Box < ast :: Item >, 1 > { SmallVec :: from_elem (item , 1)}
fn flat_map_stmt (& mut self , stmt : ast :: Stmt) -> SmallVec < ast :: Stmt , 1 > { SmallVec :: from_elem (stmt , 1)}
fn visit_pat (& mut self , pat : & mut ast :: Pat) { pat . walk_mut (self)}
fn visit_ty (& mut self , ty : & mut ast :: Ty) { ty . walk_mut (self)}
fn visit_block (& mut self , block : & mut ast :: Block) {}
fn visit_crate (& mut self , krate : & mut ast :: Crate) {}
fn flat_map_assoc_item (& mut self , item : Box < ast :: AssocItem >, ctxt : ast :: visit :: AssocCtxt) -> SmallVec < Box < ast :: AssocItem >, 1 > { SmallVec :: from_elem (item , 1)}
fn flat_map_foreign_item (& mut self , item : Box < ast :: ForeignItem >) -> SmallVec < Box < ast :: ForeignItem >, 1 > { SmallVec :: from_elem (item , 1)}
fn flat_map_variant (& mut self , variant : ast :: Variant) -> SmallVec < ast :: Variant , 1 > { SmallVec :: from_elem (variant , 1)}
fn flat_map_where_predicate (& mut self , predicate : ast :: WherePredicate) -> SmallVec < ast :: WherePredicate , 1 > { SmallVec :: from_elem (predicate , 1)}
fn flat_map_field_def (& mut self , field_def : ast :: FieldDef) -> SmallVec < ast :: FieldDef , 1 > { SmallVec :: from_elem (field_def , 1)}
fn flat_map_pat_field (& mut self , pat_field : ast :: PatField) -> SmallVec < ast :: PatField , 1 > { SmallVec :: from_elem (pat_field , 1)}
fn flat_map_expr_field (& mut self , expr_field : ast :: ExprField) -> SmallVec < ast :: ExprField , 1 > { SmallVec :: from_elem (expr_field , 1)}
fn flat_map_param (& mut self , param : ast :: Param) -> SmallVec < ast :: Param , 1 > { SmallVec :: from_elem (param , 1)}
fn flat_map_generic_param (& mut self , generic_param : ast :: GenericParam) -> SmallVec < ast :: GenericParam , 1 > { SmallVec :: from_elem (generic_param , 1)}
fn flat_map_arm (& mut self , arm : ast :: Arm) -> SmallVec < ast :: Arm , 1 > { SmallVec :: from_elem (arm , 1)}
}