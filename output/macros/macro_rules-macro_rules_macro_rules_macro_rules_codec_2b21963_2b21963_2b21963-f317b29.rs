macro_rules ! impl_arena_copy_decoder { (<$ tcx : tt > $ ($ ty : ty ,) *) => { $ (impl <'tcx , D : TyDecoder <'tcx >> RefDecodable <'tcx , D > for $ ty { #[inline] fn decode (decoder : & mut D) -> &'tcx Self { decoder . interner () . arena . alloc (Decodable :: decode (decoder))}
} impl <'tcx , D : TyDecoder <'tcx >> RefDecodable <'tcx , D > for [$ ty] { #[inline] fn decode (decoder : & mut D) -> &'tcx Self { decoder . interner () . arena . alloc_from_iter (< Vec < _ > as Decodable < D >>:: decode (decoder))}
}) *}
; }