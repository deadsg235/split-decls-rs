INSIDE_FUNCTION ! { fn expand_invoc (& mut self , invoc : Invocation , ext : & dyn SyntaxExtensionTrait ,) -> ExpandResult < AstFragment , Invocation > { let recursion_limit = match self . cx . reduced_recursion_limit { Some ((limit , _)) => limit , None => self . cx . ecfg . recursion_limit ,}
; if ! recursion_limit . value_within_limit (self . cx . current_expansion . depth) { let guar = match self . cx . reduced_recursion_limit { Some ((_ , guar)) => guar , None => self . error_recursion_limit_reached () ,}
; self . cx . reduced_recursion_limit = Some ((recursion_limit / 2 , guar)) ; return ExpandResult :: Ready (invoc . fragment_kind . dummy (invoc . span () , guar)) ;}
let macro_stats = self . cx . sess . opts . unstable_opts . macro_stats ; let (fragment_kind , span) = (invoc . fragment_kind , invoc . span ()) ; ExpandResult :: Ready (match invoc . kind { InvocationKind :: Bang { mac , span}
=> { if let Some (expander) = ext . as_bang () . and_then (| e | e . downcast_ref ::< Arc < dyn BangProcMacro + Send + Sync >> ()) { match expander . expand (self . cx , span , mac . args . tokens . clone ()) { Ok (tok_result) => { let fragment = self . parse_ast_fragment (tok_result , fragment_kind , & mac . path , span) ; if macro_stats { update_bang_macro_stats (self . cx , fragment_kind , span , mac , & fragment ,) ;}
fragment}
Err (guar) => return ExpandResult :: Ready (fragment_kind . dummy (span , guar)) ,}
} else if let Some (expander) = ext . as_legacy_bang () . and_then (| e | e . downcast_ref ::< Arc < MacroExpanderFn >> ()) { let tok_result = match expander . expand (self . cx , span , mac . args . tokens . clone ()) { ExpandResult :: Ready (tok_result) => tok_result , ExpandResult :: Retry (_) => { return ExpandResult :: Retry (Invocation { kind : InvocationKind :: Bang { mac , span}
, .. invoc }) ;}
} ; if let Some (fragment) = fragment_kind . make_from (tok_result) { if macro_stats { update_bang_macro_stats (self . cx , fragment_kind , span , mac , & fragment) ;}
fragment}
else { let guar = self . error_wrong_fragment_kind (fragment_kind , & mac , span) ; fragment_kind . dummy (span , guar)}
} else { unreachable ! () ;}
} InvocationKind :: Attr { attr , pos , mut item , derives}
=> { if let Some (expander) = ext . as_attr () . and_then (| e | e . downcast_ref ::< Arc < dyn AttrProcMacro + Send + Sync >> ()) { self . gate_proc_macro_input (& item) ; self . gate_proc_macro_attr_item (span , & item) ; let tokens = match & item { Annotatable :: Crate (krate) => { rustc_parse :: fake_token_stream_for_crate (& self . cx . sess . psess , krate)}
Annotatable :: Item (item_inner) if matches ! (attr . style , AttrStyle :: Inner) && matches ! (item_inner . kind , ItemKind :: Mod (_ , _ , ModKind :: Unloaded | ModKind :: Loaded (_ , Inline :: No { ..}
, _) ,)) => { rustc_parse :: fake_token_stream_for_item (& self . cx . sess . psess , item_inner)}
_ => item . to_tokens () ,}
; let attr_item = attr . get_normal_item () ; let safety = attr_item . unsafety ; if let AttrArgs :: Eq { ..}
= attr_item . args { self . cx . dcx () . emit_err (UnsupportedKeyValue { span }) ;}
let inner_tokens = attr_item . args . inner_tokens () ; match expander . expand_with_safety (self . cx , safety , span , inner_tokens , tokens) { Ok (tok_result) => { let fragment = self . parse_ast_fragment (tok_result , fragment_kind , & attr_item . path , span ,) ; if macro_stats { update_attr_macro_stats (self . cx , fragment_kind , span , & attr_item . path , & attr , item , & fragment ,) ;}
fragment}
Err (guar) => return ExpandResult :: Ready (fragment_kind . dummy (span , guar)) ,}
} else if let Some (expander) = ext . as_legacy_derive () . and_then (| e | e . downcast_ref ::< Arc < dyn MultiItemModifier + Send + Sync >> ()) { match validate_attr :: parse_meta (& self . cx . sess . psess , & attr) { Ok (meta) => { let item_clone = macro_stats . then (|| item . clone ()) ; let items = match expander . expand (self . cx , span , & meta , item , false) { ExpandResult :: Ready (items) => items , ExpandResult :: Retry (item) => { return ExpandResult :: Retry (Invocation { kind : InvocationKind :: Attr { attr , pos , item , derives}
, .. invoc }) ;}
} ; if matches ! (fragment_kind , AstFragmentKind :: Expr | AstFragmentKind :: MethodReceiverExpr) && items . is_empty () { let guar = self . cx . dcx () . emit_err (RemoveExprNotSupported { span }) ; fragment_kind . dummy (span , guar)}
else { let fragment = fragment_kind . expect_from_annotatables (items . into_iter ()) ; if macro_stats { update_attr_macro_stats (self . cx , fragment_kind , span , & meta . path , & attr , item_clone . unwrap () , & fragment ,) ;}
fragment}
} Err (err) => { let _guar = err . emit () ; fragment_kind . expect_from_annotatables (iter :: once (item))}
}}
else if ext . get_macro_kind () . get_name () == "non_macro_attr" { self . cx . expanded_inert_attrs . mark (& attr) ; item . visit_attrs (| attrs | attrs . insert (pos , attr)) ; fragment_kind . expect_from_annotatables (iter :: once (item))}
else { unreachable ! () ;}
} InvocationKind :: Derive { path , item , is_const}
=> match ext . get_macro_kind () . get_name () . as_str () { "derive" | "legacy_derive" => { let expander = if ext . get_macro_kind () . get_name () == "derive" { ext . as_derive () . and_then (| e | e . downcast_ref ::< Arc < dyn MultiItemModifier + Send + Sync >> ()) . unwrap ()}
else { ext . as_legacy_derive () . and_then (| e | e . downcast_ref ::< Arc < dyn MultiItemModifier + Send + Sync >> ()) . unwrap ()}
; if ext . get_macro_kind () . get_name () == "derive" { self . gate_proc_macro_input (& item) ;}
let meta = ast :: MetaItem { unsafety : ast :: Safety :: Default , kind : MetaItemKind :: Word , span , path ,}
; let items = match expander . expand (self . cx , span , & meta , item , is_const) { ExpandResult :: Ready (items) => items , ExpandResult :: Retry (item) => { return ExpandResult :: Retry (Invocation { kind : InvocationKind :: Derive { path : meta . path , item , is_const}
, .. invoc }) ;}
} ; let fragment = fragment_kind . expect_from_annotatables (items . into_iter ()) ; if macro_stats { update_derive_macro_stats (self . cx , fragment_kind , span , & meta . path , & fragment ,) ;}
fragment}
"macro_rules" => { let expander = DownCaseArcMacroRulesMacroExpanderDRT !; if is_const { let guar = self . cx . dcx () . span_err (span , "macro `derive` does not support const derives") ; return ExpandResult :: Ready (fragment_kind . dummy (span , guar)) ;}
let body = item . to_tokens () ; match expander . expand_derive (self . cx , span , & body) { Ok (tok_result) => { let fragment = self . parse_ast_fragment (tok_result , fragment_kind , & path , span) ; if macro_stats { update_derive_macro_stats (self . cx , fragment_kind , span , & path , & fragment ,) ;}
fragment}
Err (guar) => return ExpandResult :: Ready (fragment_kind . dummy (span , guar)) ,}
} _ => unreachable ! () ,}
, InvocationKind :: GlobDelegation { item , of_trait}
=> { let AssocItemKind :: DelegationMac (deleg) = & item . kind else { unreachable ! ()}
; let suffixes = match ext . get_macro_kind () . get_name () . as_str () { "glob_delegation" => { let expander = ext . as_glob_delegation () . and_then (| e | e . downcast_ref ::< Arc < dyn GlobDelegationExpander + Send + Sync >> ()) . unwrap () ; match expander . expand (self . cx) { ExpandResult :: Ready (suffixes) => suffixes , ExpandResult :: Retry (()) => { return ExpandResult :: Retry (Invocation { kind : InvocationKind :: GlobDelegation { item , of_trait}
, .. invoc }) ;}
}}
, "bang" => { let msg = "expanded a dummy glob delegation" ; let guar = self . cx . dcx () . span_delayed_bug (span , msg) ; return ExpandResult :: Ready (fragment_kind . dummy (span , guar)) ;}
_ => unreachable ! () ,}
; type Node = AstNodeWrapper < Box < ast :: AssocItem >, ImplItemTag >; let single_delegations = build_single_delegations ::< Node > (self . cx , deleg , & item , & suffixes , item . span , true ,) ; fragment_kind . expect_from_annotatables (single_delegations . map (| item | { Annotatable :: AssocItem (Box :: new (item) , AssocCtxt :: Impl { of_trait }) }))}
})}
}