macro_rules ! impl_arena_allocatable_decoders { ([$ ($ a : tt $ name : ident : $ ty : ty ,) *]) => { $ (impl_arena_allocatable_decoder ! ($ a [$ name : $ ty]) ;) *}
}