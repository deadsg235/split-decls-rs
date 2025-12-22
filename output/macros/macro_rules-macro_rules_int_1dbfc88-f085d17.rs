macro_rules ! from_scalar_int_for_x { ($ ($ ty : ty) ,*) => { $ (impl From < ScalarInt > for $ ty { #[inline] fn from (int : ScalarInt) -> Self { int . to_uint (Size :: from_bytes (size_of ::<$ ty > ())) . try_into () . unwrap ()}
}) *}
}