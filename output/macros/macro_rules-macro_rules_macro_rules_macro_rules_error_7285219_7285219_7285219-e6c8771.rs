#[macro_export] macro_rules ! err_unsup_format { ($ ($ tt : tt) *) => { $ crate :: err_unsup ! (Unsupported (format ! ($ ($ tt) *)))}
; }