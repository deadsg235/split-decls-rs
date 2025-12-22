#[macro_export] macro_rules ! throw_ub_custom { ($ ($ tt : tt) *) => { do yeet $ crate :: err_ub_custom ! ($ ($ tt) *)}
; }