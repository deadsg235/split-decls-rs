macro_rules ! declare_hooks { ($ ($ (#[$ attr : meta]) * hook $ name : ident ($ ($ arg : ident : $ K : ty) ,*) -> $ V : ty ;) *) => { impl <'tcx > TyCtxt <'tcx > { $ ($ (#[$ attr]) * #[inline (always)] pub fn $ name (self , $ ($ arg : $ K ,) *) -> $ V { (self . hooks .$ name) (self , $ ($ arg ,) *) }) *}
pub struct Providers { $ (pub $ name : for <'tcx > fn (TyCtxt <'tcx >, $ ($ arg : $ K ,) *) -> $ V ,) *}
impl Default for Providers { fn default () -> Self { Providers { $ ($ name : | _ , $ ($ arg ,) *| default_hook (stringify ! ($ name) , & ($ ($ arg ,) *))) ,*}
}}
impl Copy for Providers {}
impl Clone for Providers { fn clone (& self) -> Self { * self}
}}
; }