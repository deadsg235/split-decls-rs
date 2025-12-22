make_invocation_mut_visitor_impl ! { InvocationCollector , fn visit_expr_box (& mut self , expr : & mut Box < ast :: Expr >) {}
fn visit_pat_box (& mut self , pat : & mut Box < ast :: Pat >) {}
fn visit_ty_box (& mut self , ty : & mut Box < ast :: Ty >) {}
}