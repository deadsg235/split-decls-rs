macro_rules ! maybe_panic { ($ ($ arg : tt) *) => ({ let _err = ($ ($ arg) *) ; if cfg ! (debug_assertions) { panic ! ("{:?}" , _err) ;}
else { error ! ("Internal Hyper error, please report {:?}" , _err) ; return Err (Parse :: Internal)}
}) }