macro_rules ! define_tests { ($ ($ name : ident $ kind : ident $ variant : ident { $ ($ init : tt) * }) *) => { $ (#[test] fn $ name () { let unambig = $ kind ::$ variant ::<'_ , () > { $ ($ init) *}
; let unambig_to_ambig = unsafe { std :: mem :: transmute ::< _ , $ kind <'_ , AmbigArg >> (unambig)}
; assert ! (matches ! (& unambig_to_ambig , &$ kind ::$ variant { $ ($ init) * })) ; let ambig_to_unambig = unsafe { std :: mem :: transmute ::< _ , $ kind <'_ , () >> (unambig_to_ambig)}
; assert ! (matches ! (& ambig_to_unambig , &$ kind ::$ variant { $ ($ init) * })) ; }) *}
; }