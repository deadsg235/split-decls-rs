ffi_fn ! { #[doc = " Set whether to use HTTP2."] #[doc = ""] #[doc = " Pass `0` to disable, `1` to enable."] fn hyper_clientconn_options_http2 (opts : * mut hyper_clientconn_options , enabled : c_int) -> hyper_code { #[cfg (feature = "http2")] { let opts = non_null ! { & mut * opts ?= hyper_code :: HYPERE_INVALID_ARG}
; opts . http2 = enabled != 0 ; hyper_code :: HYPERE_OK}
#[cfg (not (feature = "http2"))] { drop (opts) ; drop (enabled) ; hyper_code :: HYPERE_FEATURE_NOT_ENABLED}
} }