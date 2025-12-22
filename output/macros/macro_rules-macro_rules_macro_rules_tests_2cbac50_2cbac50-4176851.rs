macro_rules ! test_type { ($ ty : ty) => { { struct Foo { a : $ ty , b : $ ty ,}
impl < CTX > HashStable < CTX > for Foo { fn hash_stable (& self , hcx : & mut CTX , hasher : & mut StableHasher) { self . a . hash_stable (hcx , hasher) ; self . b . hash_stable (hcx , hasher) ;}
} #[allow (overflowing_literals)] let mut item = Foo { a : 0xFF , b : 0xFF_FF}
; let hash_a = hash (& item) ; std :: mem :: swap (& mut item . a , & mut item . b) ; let hash_b = hash (& item) ; assert_ne ! (hash_a , hash_b , "The hash stayed the same after values were swapped for type `{}`!" , stringify ! ($ ty)) ;}
} ; }