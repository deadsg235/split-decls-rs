macro_rules ! impl_has_tokens { ($ ($ T : ty) ,+ $ (,) ?) => { $ (impl HasTokens for $ T { fn tokens (& self) -> Option <& LazyAttrTokenStream > { self . tokens . as_ref ()}
fn tokens_mut (& mut self) -> Option <& mut Option < LazyAttrTokenStream >> { Some (& mut self . tokens)}
}) +}
; }