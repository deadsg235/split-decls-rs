#[doc = " If `separate_provide_extern`, then the key can be projected to its"] #[doc = " local key via `<$K as AsLocalKey>::LocalKey`."] macro_rules ! local_key_if_separate_extern { ([] $ ($ K : tt) *) => { $ ($ K) *}
; ([(separate_provide_extern) $ ($ rest : tt) *] $ ($ K : tt) *) => { <$ ($ K) * as AsLocalKey >:: LocalKey}
; ([$ other : tt $ ($ modifiers : tt) *] $ ($ K : tt) *) => { local_key_if_separate_extern ! ([$ ($ modifiers) *] $ ($ K) *)}
; }