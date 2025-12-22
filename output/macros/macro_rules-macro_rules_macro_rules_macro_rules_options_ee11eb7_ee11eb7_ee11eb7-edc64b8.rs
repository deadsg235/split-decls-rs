macro_rules ! tmod_enum_opt { ($ struct_name : ident , $ tmod_enum_name : ident , $ opt : ident , $ v : ident) => { Some (OptionsTargetModifiers ::$ struct_name ($ tmod_enum_name ::$ opt))}
; ($ struct_name : ident , $ tmod_enum_name : ident , $ opt : ident ,) => { None}
; }