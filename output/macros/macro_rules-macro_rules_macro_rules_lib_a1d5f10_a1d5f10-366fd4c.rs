macro_rules ! check_str_lit_len { ($ ($ s : literal) ,* $ (,) ?) => { $ ({ let lit = stringify ! ($ s) ; assert_eq ! (find_str_lit_len (lit) , Some (lit . len ())) ; }) *}
}