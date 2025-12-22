macro_rules ! impl_dyn_send { ($ ($ ($ attr : meta) * [$ ty : ty where $ ($ generics2 : tt) *]) *) => { $ (unsafe impl <$ ($ generics2) *> DynSend for $ ty { }) *}
; }