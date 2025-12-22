macro_rules ! basic_blocks_iter { ($ body : ident , mut , $ invalidate : tt) => { basic_blocks ! ($ body , mut , $ invalidate) . iter_enumerated_mut ()}
; ($ body : ident ,) => { basic_blocks ! ($ body ,) . iter_enumerated ()}
; }