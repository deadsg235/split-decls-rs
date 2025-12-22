macro_rules ! visit_visitable { ($ visitor : expr , $ ($ expr : expr) ,* $ (,) ?) => { { $ (try_visit ! (Visitable :: visit ($ expr , $ visitor , ())) ;) *}
} ; }