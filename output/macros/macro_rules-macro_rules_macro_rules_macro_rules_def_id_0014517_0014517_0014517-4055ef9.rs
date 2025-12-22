macro_rules ! typed_def_id { ($ Name : ident , $ LocalName : ident) => { #[derive (Debug , Clone , Copy , PartialEq , Eq , Hash , Encodable , Decodable , HashStable_Generic)] pub struct $ Name (DefId) ; impl $ Name { #[inline] pub const fn new_unchecked (def_id : DefId) -> Self { Self (def_id)}
#[inline] pub fn to_def_id (self) -> DefId { self . into ()}
#[inline] pub fn is_local (self) -> bool { self . 0 . is_local ()}
#[inline] pub fn as_local (self) -> Option <$ LocalName > { self . 0 . as_local () . map ($ LocalName :: new_unchecked)}
} impl From <$ LocalName > for $ Name { #[inline] fn from (local : $ LocalName) -> Self { Self (local . 0 . to_def_id ())}
} impl From <$ Name > for DefId { #[inline] fn from (typed : $ Name) -> Self { typed . 0}
} #[derive (Debug , Clone , Copy , PartialEq , Eq , Hash , Encodable , Decodable , HashStable_Generic)] pub struct $ LocalName (LocalDefId) ; impl ! Ord for $ LocalName {}
impl ! PartialOrd for $ LocalName {}
impl $ LocalName { #[inline] pub const fn new_unchecked (def_id : LocalDefId) -> Self { Self (def_id)}
#[inline] pub fn to_def_id (self) -> DefId { self . 0 . into ()}
#[inline] pub fn to_local_def_id (self) -> LocalDefId { self . 0}
} impl From <$ LocalName > for LocalDefId { #[inline] fn from (typed : $ LocalName) -> Self { typed . 0}
} impl From <$ LocalName > for DefId { #[inline] fn from (typed : $ LocalName) -> Self { typed . 0 . into ()}
}}
; }