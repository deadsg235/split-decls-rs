macro_rules ! define_callbacks { ($ ($ (#[$ attr : meta]) * [$ ($ modifiers : tt) *] fn $ name : ident ($ ($ K : tt) *) -> $ V : ty ,) *) => { #[allow (unused_lifetimes)] pub mod queries { $ (pub mod $ name { use super :: super ::*; pub type Key <'tcx > = $ ($ K) *; pub type Value <'tcx > = $ V ; pub type LocalKey <'tcx > = local_key_if_separate_extern ! ([$ ($ modifiers) *] $ ($ K) *) ; #[doc = " This type alias specifies the type returned from query providers and the type"] #[doc = " used for decoding. For regular queries this is the declared returned type `V`,"] #[doc = " but `arena_cache` will use `<V as ArenaCached>::Provided` instead."] pub type ProvidedValue <'tcx > = query_if_arena ! ([$ ($ modifiers) *] (<$ V as $ crate :: query :: arena_cached :: ArenaCached <'tcx >>:: Provided) ($ V)) ; #[doc = " This function takes `ProvidedValue` and coverts it to an erased `Value` by"] #[doc = " allocating it on an arena if the query has the `arena_cache` modifier. The"] #[doc = " value is then erased and returned. This will happen when computing the query"] #[doc = " using a provider or decoding a stored result."] #[inline (always)] pub fn provided_to_erased <'tcx > (_tcx : TyCtxt <'tcx >, value : ProvidedValue <'tcx >,) -> Erase < Value <'tcx >> { erase (query_if_arena ! ([$ ($ modifiers) *] { use $ crate :: query :: arena_cached :: ArenaCached ; if mem :: needs_drop ::<<$ V as ArenaCached <'tcx >>:: Allocated > () { <$ V as ArenaCached >:: alloc_in_arena (| v | _tcx . query_system . arenas .$ name . alloc (v) , value ,)}
else { <$ V as ArenaCached >:: alloc_in_arena (| v | _tcx . arena . dropless . alloc (v) , value ,)}
} (value)))}
pub type Storage <'tcx > = <$ ($ K) * as keys :: Key >:: Cache < Erase <$ V >>; #[cfg (target_pointer_width = "64")] const _ : () = { if size_of ::< Key <'static >> () > 88 { panic ! ("{}" , concat ! ("the query `" , stringify ! ($ name) , "` has a key type `" , stringify ! ($ ($ K) *) , "` that is too large")) ;}
} ; #[cfg (target_pointer_width = "64")] #[cfg (not (feature = "rustc_randomized_layouts"))] const _ : () = { if size_of ::< Value <'static >> () > 64 { panic ! ("{}" , concat ! ("the query `" , stringify ! ($ name) , "` has a value type `" , stringify ! ($ V) , "` that is too large")) ;}
} ; }) *}
pub struct QueryArenas <'tcx > { $ ($ (#[$ attr]) * pub $ name : query_if_arena ! ([$ ($ modifiers) *] (TypedArena <<$ V as $ crate :: query :: arena_cached :: ArenaCached <'tcx >>:: Allocated >) ()) ,) *}
impl Default for QueryArenas <'_ > { fn default () -> Self { Self { $ ($ name : query_if_arena ! ([$ ($ modifiers) *] (Default :: default ()) ()) ,) *}
}}
#[derive (Default)] pub struct QueryCaches <'tcx > { $ ($ (#[$ attr]) * pub $ name : queries ::$ name :: Storage <'tcx >,) *}
impl <'tcx > TyCtxtEnsureOk <'tcx > { $ ($ (#[$ attr]) * #[inline (always)] pub fn $ name (self , key : query_helper_param_ty ! ($ ($ K) *) ,) -> ensure_ok_result ! ([$ ($ modifiers) *]) { query_ensure_select ! ([$ ($ modifiers) *] self . tcx , self . tcx . query_system . fns . engine .$ name , & self . tcx . query_system . caches .$ name , key . into_query_param () , false ,) }) *}
impl <'tcx > TyCtxtEnsureDone <'tcx > { $ ($ (#[$ attr]) * #[inline (always)] pub fn $ name (self , key : query_helper_param_ty ! ($ ($ K) *)) { crate :: query :: inner :: query_ensure (self . tcx , self . tcx . query_system . fns . engine .$ name , & self . tcx . query_system . caches .$ name , key . into_query_param () , true ,) ; }) *}
impl <'tcx > TyCtxt <'tcx > { $ ($ (#[$ attr]) * #[inline (always)] #[must_use] pub fn $ name (self , key : query_helper_param_ty ! ($ ($ K) *)) -> $ V { self . at (DUMMY_SP) .$ name (key) }) *}
impl <'tcx > TyCtxtAt <'tcx > { $ ($ (#[$ attr]) * #[inline (always)] pub fn $ name (self , key : query_helper_param_ty ! ($ ($ K) *)) -> $ V { restore ::<$ V > (crate :: query :: inner :: query_get_at (self . tcx , self . tcx . query_system . fns . engine .$ name , & self . tcx . query_system . caches .$ name , self . span , key . into_query_param () ,)) }) *}
pub struct DynamicQueries <'tcx > { $ (pub $ name : DynamicQuery <'tcx , queries ::$ name :: Storage <'tcx >>,) *}
#[derive (Default)] pub struct QueryStates <'tcx > { $ (pub $ name : QueryState <$ ($ K) *, QueryStackDeferred <'tcx >>,) *}
pub struct Providers { $ (pub $ name : for <'tcx > fn (TyCtxt <'tcx >, queries ::$ name :: LocalKey <'tcx >,) -> queries ::$ name :: ProvidedValue <'tcx >,) *}
pub struct ExternProviders { $ (pub $ name : separate_provide_extern_decl ! ([$ ($ modifiers) *] [$ name]) ,) *}
impl Default for Providers { fn default () -> Self { Providers { $ ($ name : | _ , key | $ crate :: query :: plumbing :: default_query (stringify ! ($ name) , & key)) ,*}
}}
impl Default for ExternProviders { fn default () -> Self { ExternProviders { $ ($ name : separate_provide_extern_default ! ([$ ($ modifiers) *] [$ name]) ,) *}
}}
impl Copy for Providers {}
impl Clone for Providers { fn clone (& self) -> Self { * self}
} impl Copy for ExternProviders {}
impl Clone for ExternProviders { fn clone (& self) -> Self { * self}
} pub struct QueryEngine { $ (pub $ name : for <'tcx > fn (TyCtxt <'tcx >, Span , queries ::$ name :: Key <'tcx >, QueryMode ,) -> Option < Erase <$ V >>,) *}
} ; }