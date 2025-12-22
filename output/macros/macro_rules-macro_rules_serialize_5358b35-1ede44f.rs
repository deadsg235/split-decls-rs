macro_rules ! direct_serialize_impls { ($ ($ ty : ident $ emit_method : ident $ read_method : ident) ,*) => { $ (impl < S : Encoder > Encodable < S > for $ ty { fn encode (& self , s : & mut S) { s .$ emit_method (* self) ;}
} impl < D : Decoder > Decodable < D > for $ ty { fn decode (d : & mut D) -> $ ty { d .$ read_method ()}
}) *}
}