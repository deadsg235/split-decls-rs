macro_rules ! info { ($ ($ arg : tt) *) => { #[cfg (feature = "tracing")] { tracing :: info ! ($ ($ arg) +) ;}
} }