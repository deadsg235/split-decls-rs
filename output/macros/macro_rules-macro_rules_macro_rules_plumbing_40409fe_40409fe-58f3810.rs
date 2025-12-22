macro_rules ! separate_provide_extern_decl { ([] [$ name : ident]) => { ()}
; ([(separate_provide_extern) $ ($ rest : tt) *] [$ name : ident]) => { for <'tcx > fn (TyCtxt <'tcx >, queries ::$ name :: Key <'tcx >,) -> queries ::$ name :: ProvidedValue <'tcx >}
; ([$ other : tt $ ($ modifiers : tt) *] [$ ($ args : tt) *]) => { separate_provide_extern_decl ! ([$ ($ modifiers) *] [$ ($ args) *])}
; }