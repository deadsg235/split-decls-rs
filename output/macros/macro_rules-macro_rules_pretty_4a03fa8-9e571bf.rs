macro_rules ! define_print { (($ self : ident , $ p : ident) : $ ($ ty : ty $ print : block) +) => { $ (impl <'tcx , P : PrettyPrinter <'tcx >> Print <'tcx , P > for $ ty { fn print (&$ self , $ p : & mut P) -> Result < () , PrintError > { let _ : () = $ print ; Ok (())}
}) +}
; }