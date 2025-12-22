macro_rules ! thir_with_elements { ($ ($ name : ident : $ id : ty => $ value : ty => $ format : literal ,) *) => { $ (newtype_index ! { #[derive (HashStable)] #[debug_format = $ format] pub struct $ id {}
}) * #[doc = " A container for a THIR body."] #[doc = ""] #[doc = " This can be indexed directly by any THIR index (e.g. [`ExprId`])."] #[derive (Debug , HashStable , Clone)] pub struct Thir <'tcx > { pub body_type : BodyTy <'tcx >, $ (pub $ name : IndexVec <$ id , $ value >,) *}
impl <'tcx > Thir <'tcx > { pub fn new (body_type : BodyTy <'tcx >) -> Thir <'tcx > { Thir { body_type , $ ($ name : IndexVec :: new () ,) *}
}}
$ (impl <'tcx > Index <$ id > for Thir <'tcx > { type Output = $ value ; fn index (& self , index : $ id) -> & Self :: Output { & self .$ name [index]}
}) *}
}