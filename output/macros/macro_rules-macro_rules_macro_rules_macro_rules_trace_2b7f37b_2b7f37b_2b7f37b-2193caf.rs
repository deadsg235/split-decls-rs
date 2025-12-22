macro_rules ! info_span { ($ ($ arg : tt) *) => { { #[cfg (feature = "tracing")] { let _span = tracing :: info_span ! ($ ($ arg) +) ; _span . entered ()}
}}
}