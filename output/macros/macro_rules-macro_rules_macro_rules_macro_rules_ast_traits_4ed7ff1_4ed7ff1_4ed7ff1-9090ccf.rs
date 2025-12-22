macro_rules ! impl_has_attrs_none { ($ ($ T : ty) ,+ $ (,) ?) => { $ (impl HasAttrs for $ T { const SUPPORTS_CUSTOM_INNER_ATTRS : bool = false ; fn attrs (& self) -> & [Attribute] { & []}
fn visit_attrs (& mut self , _f : impl FnOnce (& mut AttrVec)) {}
}) +}
; }