macro_rules ! extra_body_methods { (mut) => { fn visit_body_preserves_cfg (& mut self , body : & mut Body <'tcx >) { self . super_body_preserves_cfg (body) ;}
fn super_body_preserves_cfg (& mut self , body : & mut Body <'tcx >) { super_body ! (self , body , mut , false) ;}
} ; () => {}
; }