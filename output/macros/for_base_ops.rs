for_base_ops ! { T = (f32 , f64) ; type Lhs = Simd < T , N >; type Rhs = Simd < T , N >; type Output = Self ; impl Add :: add { unsafe_base { simd_add}
} impl Mul :: mul { unsafe_base { simd_mul}
} impl Sub :: sub { unsafe_base { simd_sub}
} impl Div :: div { unsafe_base { simd_div}
} impl Rem :: rem { unsafe_base { simd_rem}
} }