ffi_fn ! { #[doc = " Set whether header order is preserved."] #[doc = ""] #[doc = " Pass `0` to allow reordering (default), `1` to retain original ordering."] fn hyper_clientconn_options_set_preserve_header_order (opts : * mut hyper_clientconn_options , enabled : c_int) { let opts = non_null ! { & mut * opts ?= ()}
; opts . http1_preserve_header_order = enabled != 0 ;}
}