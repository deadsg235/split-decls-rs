macro_rules ! query_if_arena { ([] $ arena : tt $ no_arena : tt) => { $ no_arena}
; ([(arena_cache) $ ($ rest : tt) *] $ arena : tt $ no_arena : tt) => { $ arena}
; ([$ other : tt $ ($ modifiers : tt) *] $ ($ args : tt) *) => { query_if_arena ! ([$ ($ modifiers) *] $ ($ args) *)}
; }