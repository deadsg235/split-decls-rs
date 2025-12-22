macro_rules ! from_scalar_int_for_x_signed { ($ ($ ty : ty) ,*) => { $ (impl From < ScalarInt > for $ ty { #[inline] fn from (int : ScalarInt) -> Self { int . to_int (Size :: from_bytes (size_of ::<$ ty > ())) . try_into () . unwrap ()}
}) *}
}