macro_rules ! hash_substruct { ($ opt_name : ident , $ opt_expr : expr , $ error_format : expr , $ for_crate_hash : expr , $ hasher : expr , [UNTRACKED]) => { {}
} ; ($ opt_name : ident , $ opt_expr : expr , $ error_format : expr , $ for_crate_hash : expr , $ hasher : expr , [TRACKED]) => { {}
} ; ($ opt_name : ident , $ opt_expr : expr , $ error_format : expr , $ for_crate_hash : expr , $ hasher : expr , [TRACKED_NO_CRATE_HASH]) => { {}
} ; ($ opt_name : ident , $ opt_expr : expr , $ error_format : expr , $ for_crate_hash : expr , $ hasher : expr , [SUBSTRUCT]) => { use crate :: config :: dep_tracking :: DepTrackingHash ; $ opt_expr . dep_tracking_hash ($ for_crate_hash , $ error_format) . hash ($ hasher , $ error_format , $ for_crate_hash ,) ;}
; }