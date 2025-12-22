macro_rules ! add { ($ name : expr , $ value : expr) => { adder ($ name . into () , $ value . into_diag_arg (& mut None)) ;}
; }