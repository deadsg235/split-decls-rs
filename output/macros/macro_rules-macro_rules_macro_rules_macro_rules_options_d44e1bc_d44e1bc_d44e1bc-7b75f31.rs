macro_rules ! gather_tmods_top_level { ($ _opt_name : ident , $ opt_expr : expr , $ mods : expr , $ tmod_vals : expr , [SUBSTRUCT $ substruct_enum : ident]) => { $ opt_expr . gather_target_modifiers ($ mods , $ tmod_vals) ;}
; ($ opt_name : ident , $ opt_expr : expr , $ mods : expr , $ tmod_vals : expr , [$ non_substruct : ident TARGET_MODIFIER]) => { compile_error ! ("Top level option can't be target modifier") ;}
; ($ opt_name : ident , $ opt_expr : expr , $ mods : expr , $ tmod_vals : expr , [$ non_substruct : ident]) => {}
; }