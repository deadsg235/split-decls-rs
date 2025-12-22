macro_rules ! impls_dyn_send_neg { ($ ([$ t1 : ty $ (where $ ($ generics1 : tt) *) ?]) *) => { $ (impl $ (<$ ($ generics1) *>) ? ! DynSend for $ t1 { }) *}
; }