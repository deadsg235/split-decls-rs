macro_rules ! trivial { ($ ($ ty : ty) ,+ $ (,) ?) => { $ (impl EraseType for $ ty { type Result = [u8 ; size_of ::<$ ty > ()] ; }) *}
}