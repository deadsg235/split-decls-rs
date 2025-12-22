macro_rules ! expect_methods_self_kind { ($ ($ name : ident , $ ret_ty : ty , $ pat : pat , $ ret_val : expr ;) *) => { $ (#[track_caller] pub fn $ name (& self) -> $ ret_ty { let $ pat = & self . kind else { expect_failed (stringify ! ($ name) , self)}
; $ ret_val }) *}
}