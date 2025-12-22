define_print ! { (self , p) : ty :: FnSig <'tcx > { write ! (p , "{}" , self . safety . prefix_str ()) ?; if self . abi != ExternAbi :: Rust { write ! (p , "extern {} " , self . abi) ?;}
write ! (p , "fn") ?; p . pretty_print_fn_sig (self . inputs () , self . c_variadic , self . output ()) ?;}
ty :: TraitRef <'tcx > { write ! (p , "<{} as {}>" , self . self_ty () , self . print_only_trait_path ()) ?;}
ty :: AliasTy <'tcx > { let alias_term : ty :: AliasTerm <'tcx > = (* self) . into () ; alias_term . print (p) ?;}
ty :: AliasTerm <'tcx > { match self . kind (p . tcx ()) { ty :: AliasTermKind :: InherentTy | ty :: AliasTermKind :: InherentConst => p . pretty_print_inherent_projection (* self) ?, ty :: AliasTermKind :: ProjectionTy => { if ! (p . should_print_verbose () || with_reduced_queries ()) && p . tcx () . is_impl_trait_in_trait (self . def_id) { p . pretty_print_rpitit (self . def_id , self . args) ?;}
else { p . print_def_path (self . def_id , self . args) ?;}
} ty :: AliasTermKind :: FreeTy | ty :: AliasTermKind :: FreeConst | ty :: AliasTermKind :: OpaqueTy | ty :: AliasTermKind :: UnevaluatedConst | ty :: AliasTermKind :: ProjectionConst => { p . print_def_path (self . def_id , self . args) ?;}
}}
ty :: TraitPredicate <'tcx > { self . trait_ref . self_ty () . print (p) ?; write ! (p , ": ") ?; if let ty :: PredicatePolarity :: Negative = self . polarity { write ! (p , "!") ?;}
self . trait_ref . print_trait_sugared () . print (p) ?;}
ty :: HostEffectPredicate <'tcx > { let constness = match self . constness { ty :: BoundConstness :: Const => { "const"}
ty :: BoundConstness :: Maybe => { "[const]"}
} ; self . trait_ref . self_ty () . print (p) ?; write ! (p , ": {constness} ") ?; self . trait_ref . print_trait_sugared () . print (p) ?;}
ty :: TypeAndMut <'tcx > { write ! (p , "{}" , self . mutbl . prefix_str ()) ?; self . ty . print (p) ?;}
ty :: ClauseKind <'tcx > { match * self { ty :: ClauseKind :: Trait (ref data) => data . print (p) ?, ty :: ClauseKind :: RegionOutlives (predicate) => predicate . print (p) ?, ty :: ClauseKind :: TypeOutlives (predicate) => predicate . print (p) ?, ty :: ClauseKind :: Projection (predicate) => predicate . print (p) ?, ty :: ClauseKind :: HostEffect (predicate) => predicate . print (p) ?, ty :: ClauseKind :: ConstArgHasType (ct , ty) => { write ! (p , "the constant `") ?; ct . print (p) ?; write ! (p , "` has type `") ?; ty . print (p) ?; write ! (p , "`") ?;}
, ty :: ClauseKind :: WellFormed (term) => { term . print (p) ?; write ! (p , " well-formed") ?;}
ty :: ClauseKind :: ConstEvaluatable (ct) => { write ! (p , "the constant `") ?; ct . print (p) ?; write ! (p , "` can be evaluated") ?;}
ty :: ClauseKind :: UnstableFeature (symbol) => { write ! (p , "feature({symbol}) is enabled") ?;}
}}
ty :: PredicateKind <'tcx > { match * self { ty :: PredicateKind :: Clause (data) => data . print (p) ?, ty :: PredicateKind :: Subtype (predicate) => predicate . print (p) ?, ty :: PredicateKind :: Coerce (predicate) => predicate . print (p) ?, ty :: PredicateKind :: DynCompatible (trait_def_id) => { write ! (p , "the trait `") ?; p . print_def_path (trait_def_id , & []) ?; write ! (p , "` is dyn-compatible") ?;}
ty :: PredicateKind :: ConstEquate (c1 , c2) => { write ! (p , "the constant `") ?; c1 . print (p) ?; write ! (p , "` equals `") ?; c2 . print (p) ?; write ! (p , "`") ?;}
ty :: PredicateKind :: Ambiguous => write ! (p , "ambiguous") ?, ty :: PredicateKind :: NormalizesTo (data) => data . print (p) ?, ty :: PredicateKind :: AliasRelate (t1 , t2 , dir) => { t1 . print (p) ?; write ! (p , " {dir} ") ?; t2 . print (p) ?;}
}}
ty :: ExistentialPredicate <'tcx > { match * self { ty :: ExistentialPredicate :: Trait (x) => x . print (p) ?, ty :: ExistentialPredicate :: Projection (x) => x . print (p) ?, ty :: ExistentialPredicate :: AutoTrait (def_id) => p . print_def_path (def_id , & []) ?,}
} ty :: ExistentialTraitRef <'tcx > { let dummy_self = Ty :: new_fresh (p . tcx () , 0) ; let trait_ref = self . with_self_ty (p . tcx () , dummy_self) ; trait_ref . print_only_trait_path () . print (p) ?;}
ty :: ExistentialProjection <'tcx > { let name = p . tcx () . associated_item (self . def_id) . name () ; let args = & self . args [p . tcx () . generics_of (self . def_id) . parent_count - 1 ..] ; p . print_path_with_generic_args (| p | write ! (p , "{name}") , args) ?; write ! (p , " = ") ?; self . term . print (p) ?;}
ty :: ProjectionPredicate <'tcx > { self . projection_term . print (p) ?; write ! (p , " == ") ?; p . reset_type_limit () ; self . term . print (p) ?;}
ty :: SubtypePredicate <'tcx > { self . a . print (p) ?; write ! (p , " <: ") ?; p . reset_type_limit () ; self . b . print (p) ?;}
ty :: CoercePredicate <'tcx > { self . a . print (p) ?; write ! (p , " -> ") ?; p . reset_type_limit () ; self . b . print (p) ?;}
ty :: NormalizesTo <'tcx > { self . alias . print (p) ?; write ! (p , " normalizes-to ") ?; p . reset_type_limit () ; self . term . print (p) ?;}
}