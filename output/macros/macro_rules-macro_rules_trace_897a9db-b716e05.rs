macro_rules ! debug_span { ($ ($ arg : tt) *) => { { #[cfg (feature = "tracing")] { let _span = tracing :: debug_span ! ($ ($ arg) +) ; _span . entered ()}
}}
}