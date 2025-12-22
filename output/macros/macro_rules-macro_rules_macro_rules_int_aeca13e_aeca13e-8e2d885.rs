macro_rules ! from_x_for_scalar_int { ($ ($ ty : ty) ,*) => { $ (impl From <$ ty > for ScalarInt { #[inline] fn from (u : $ ty) -> Self { Self { data : u128 :: from (u) , size : NonZero :: new (size_of ::<$ ty > () as u8) . unwrap () ,}
} }) *}
}