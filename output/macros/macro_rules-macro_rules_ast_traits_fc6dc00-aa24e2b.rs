macro_rules ! impl_has_node_id { ($ ($ T : ty) ,+ $ (,) ?) => { $ (impl HasNodeId for $ T { fn node_id (& self) -> NodeId { self . id}
fn node_id_mut (& mut self) -> & mut NodeId { & mut self . id}
}) +}
; }