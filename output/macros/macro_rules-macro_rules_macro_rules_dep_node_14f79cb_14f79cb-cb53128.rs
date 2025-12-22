macro_rules ! define_dep_nodes { ($ ($ (#[$ attr : meta]) * [$ ($ modifiers : tt) *] fn $ variant : ident ($ ($ K : tt) *) -> $ V : ty ,) *) => { #[macro_export] macro_rules ! make_dep_kind_array { ($ mod : ident) => { [$ ($ mod ::$ variant ()) ,*]}
;}
#[macro_export] macro_rules ! make_dep_kind_name_array { ($ mod : ident) => { vec ! { $ (*$ mod ::$ variant () . name) ,*}
} ;}
#[doc = " This enum serves as an index into arrays built by `make_dep_kind_array`."] #[allow (non_camel_case_types)] #[repr (u16)] enum DepKindDefs { $ ($ (#[$ attr]) * $ variant) ,*}
#[allow (non_upper_case_globals)] pub mod dep_kinds { use super ::*; $ (pub const $ variant : DepKind = DepKind :: new (DepKindDefs ::$ variant as u16) ;) *}
pub const DEP_KIND_VARIANTS : u16 = { let deps = & [$ (dep_kinds ::$ variant ,) *] ; let mut i = 0 ; while i < deps . len () { if i != deps [i] . as_usize () { panic ! () ;}
i += 1 ;}
deps . len () as u16}
; pub (super) fn dep_kind_from_label_string (label : & str) -> Result < DepKind , () > { match label { $ (stringify ! ($ variant) => Ok (dep_kinds ::$ variant) ,) * _ => Err (()) ,}
} #[doc = " Contains variant => str representations for constructing"] #[doc = " DepNode groups for tests."] #[allow (dead_code , non_upper_case_globals)] pub mod label_strs { $ (pub const $ variant : & str = stringify ! ($ variant) ;) *}
} ; }