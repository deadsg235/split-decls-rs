macro_rules ! hash_result { ([]) => { { Some (dep_graph :: hash_result)}
} ; ([(no_hash) $ ($ rest : tt) *]) => { { None}
} ; ([$ other : tt $ ($ modifiers : tt) *]) => { hash_result ! ([$ ($ modifiers) *])}
; }