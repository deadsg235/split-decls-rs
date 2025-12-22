macro_rules ! match_span_kind { ($ span : expr , InlineCtxt ($ span1 : ident) => $ arm1 : expr , InlineParent ($ span2 : ident) => $ arm2 : expr , PartiallyInterned ($ span3 : ident) => $ arm3 : expr , Interned ($ span4 : ident) => $ arm4 : expr ,) => { if $ span . len_with_tag_or_marker != BASE_LEN_INTERNED_MARKER { if $ span . len_with_tag_or_marker & PARENT_TAG == 0 { let $ span1 = InlineCtxt :: from_span ($ span) ; $ arm1}
else { let $ span2 = InlineParent :: from_span ($ span) ; $ arm2}
} else if $ span . ctxt_or_parent_or_marker != CTXT_INTERNED_MARKER { let $ span3 = PartiallyInterned :: from_span ($ span) ; $ arm3}
else { let $ span4 = Interned :: from_span ($ span) ; $ arm4}
} ; }