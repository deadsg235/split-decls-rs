macro_rules ! impl_has_attrs { (const SUPPORTS_CUSTOM_INNER_ATTRS : bool = $ inner : literal , $ ($ T : ty) ,+ $ (,) ?) => { $ (impl HasAttrs for $ T { const SUPPORTS_CUSTOM_INNER_ATTRS : bool = $ inner ; #[inline] fn attrs (& self) -> & [Attribute] { & self . attrs}
fn visit_attrs (& mut self , f : impl FnOnce (& mut AttrVec)) { f (& mut self . attrs)}
}) +}
; }