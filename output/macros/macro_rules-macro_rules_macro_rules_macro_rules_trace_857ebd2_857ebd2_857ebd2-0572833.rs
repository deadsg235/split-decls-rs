macro_rules ! debug { ($ ($ arg : tt) +) => { #[cfg (feature = "tracing")] { tracing :: debug ! ($ ($ arg) +) ;}
} }