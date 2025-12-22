# [proc_macro] pub fn macro_rules_missing_unsafe (_input : TokenStream) -> TokenStream { "macro_rules! make_fn {
        () => { #[no_mangle] pub fn foo() { } };
    }" . parse () . unwrap () } . sig