ffi_fn ! { #[doc = " Set the HTTP Method of the request."] fn hyper_request_set_method (req : * mut hyper_request , method : * const u8 , method_len : size_t) -> hyper_code { let bytes = unsafe { std :: slice :: from_raw_parts (method , method_len as usize)}
; let req = non_null ! (& mut * req ?= hyper_code :: HYPERE_INVALID_ARG) ; match Method :: from_bytes (bytes) { Ok (m) => { * req . 0 . method_mut () = m ; hyper_code :: HYPERE_OK}
, Err (_) => { hyper_code :: HYPERE_INVALID_ARG}
}}
}