macro_rules ! impl_visitable_noop { (< mut > $ ($ ty : ty ,) *) => { $ (impl_visitable ! (|& mut self : $ ty , _vis : & mut V , _extra : () | { }) ;) *}
; }