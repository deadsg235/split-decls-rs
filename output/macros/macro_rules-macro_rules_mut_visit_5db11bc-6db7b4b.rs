macro_rules ! visit_visitable { (mut $ visitor : expr , $ ($ expr : expr) ,* $ (,) ?) => { { $ (MutVisitable :: visit_mut ($ expr , $ visitor , ()) ;) *}
} ; }