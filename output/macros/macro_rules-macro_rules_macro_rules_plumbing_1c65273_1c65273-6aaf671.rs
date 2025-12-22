macro_rules ! separate_provide_extern_default { ([] [$ name : ident]) => { ()}
; ([(separate_provide_extern) $ ($ rest : tt) *] [$ name : ident]) => { | _ , key | $ crate :: query :: plumbing :: default_extern_query (stringify ! ($ name) , & key)}
; ([$ other : tt $ ($ modifiers : tt) *] [$ ($ args : tt) *]) => { separate_provide_extern_default ! ([$ ($ modifiers) *] [$ ($ args) *])}
; }