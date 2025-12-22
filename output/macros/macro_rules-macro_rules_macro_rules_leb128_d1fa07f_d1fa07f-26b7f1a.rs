macro_rules ! impl_read_unsigned_leb128 { ($ fn_name : ident , $ int_ty : ty) => { #[inline] pub fn $ fn_name (decoder : & mut MemDecoder <'_ >) -> $ int_ty { let byte = decoder . read_u8 () ; if (byte & 0x80) == 0 { return byte as $ int_ty ;}
let mut result = (byte & 0x7F) as $ int_ty ; let mut shift = 7 ; loop { let byte = decoder . read_u8 () ; if (byte & 0x80) == 0 { result |= (byte as $ int_ty) << shift ; return result ;}
else { result |= ((byte & 0x7F) as $ int_ty) << shift ;}
shift = shift . debug_strict_add (7) ;}
}}
; }