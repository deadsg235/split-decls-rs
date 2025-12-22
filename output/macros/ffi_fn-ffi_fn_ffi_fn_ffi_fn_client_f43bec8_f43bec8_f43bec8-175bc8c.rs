ffi_fn ! { #[doc = " Set the client background task executor."] #[doc = ""] #[doc = " This does not consume the `options` or the `exec`."] fn hyper_clientconn_options_exec (opts : * mut hyper_clientconn_options , exec : * const hyper_executor) { let opts = non_null ! { & mut * opts ?= ()}
; let exec = non_null ! { Arc :: from_raw (exec) ?= ()}
; let weak_exec = hyper_executor :: downgrade (& exec) ; std :: mem :: forget (exec) ; opts . exec = weak_exec ;}
}