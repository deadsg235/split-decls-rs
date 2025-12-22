macro_rules ! impls { ($ ($ ty : ident) *) => { $ (impl AddAssign < Delta <$ ty >> for $ ty { fn add_assign (& mut self , rhs : Delta <$ ty >) { match rhs { Delta :: Add (amt) => * self += amt , Delta :: Sub (amt) => * self -= amt ,}
} }) *}
; }