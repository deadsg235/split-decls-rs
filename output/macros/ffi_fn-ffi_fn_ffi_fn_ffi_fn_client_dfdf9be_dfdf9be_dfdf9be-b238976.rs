ffi_fn ! { #[doc = " Set whether header case is preserved."] #[doc = ""] #[doc = " Pass `0` to allow lowercase normalization (default), `1` to retain original case."] fn hyper_clientconn_options_set_preserve_header_case (opts : * mut hyper_clientconn_options , enabled : c_int) { let opts = non_null ! { & mut * opts ?= ()}
; opts . http1_preserve_header_case = enabled != 0 ;}
}