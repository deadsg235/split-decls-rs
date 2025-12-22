macro_rules ! visit_visitable_with { ($ visitor : expr , $ expr : expr , $ extra : expr $ (,) ?) => { try_visit ! (Visitable :: visit ($ expr , $ visitor , $ extra))}
; }