macro_rules ! trace_span { ($ ($ arg : tt) *) => { { #[cfg (feature = "tracing")] { let _span = tracing :: trace_span ! ($ ($ arg) +) ; _span . entered ()}
}}
}