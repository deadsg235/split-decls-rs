macro_rules ! super_body { ($ self : ident , $ body : ident , $ ($ mutability : ident , $ invalidate : tt) ?) => { let span = $ body . span ; if let Some (coroutine) = &$ ($ mutability) ? $ body . coroutine { if let Some (yield_ty) = $ (& $ mutability) ? coroutine . yield_ty { $ self . visit_ty (yield_ty , TyContext :: YieldTy (SourceInfo :: outermost (span))) ;}
if let Some (resume_ty) = $ (& $ mutability) ? coroutine . resume_ty { $ self . visit_ty (resume_ty , TyContext :: ResumeTy (SourceInfo :: outermost (span))) ;}
} for var_debug_info in &$ ($ mutability) ? $ body . var_debug_info { $ self . visit_var_debug_info (var_debug_info) ;}
for (bb , data) in basic_blocks_iter ! ($ body , $ ($ mutability , $ invalidate) ?) { $ self . visit_basic_block_data (bb , data) ;}
for scope in &$ ($ mutability) ? $ body . source_scopes { $ self . visit_source_scope_data (scope) ;}
$ self . visit_ty ($ (& $ mutability) ? $ body . return_ty () , TyContext :: ReturnTy (SourceInfo :: outermost ($ body . span))) ; for local in $ body . local_decls . indices () { $ self . visit_local_decl (local , & $ ($ mutability) ? $ body . local_decls [local]) ;}
#[allow (unused_macro_rules)] macro_rules ! type_annotations { (mut) => ($ body . user_type_annotations . iter_enumerated_mut ()) ; () => ($ body . user_type_annotations . iter_enumerated ()) ;}
for (index , annotation) in type_annotations ! ($ ($ mutability) ?) { $ self . visit_user_type_annotation (index , annotation) ;}
$ self . visit_span ($ (& $ mutability) ? $ body . span) ; if let Some (required_consts) = &$ ($ mutability) ? $ body . required_consts { for const_ in required_consts { let location = Location :: START ; $ self . visit_const_operand (const_ , location) ;}
}}
}