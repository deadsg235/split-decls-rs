ffi_fn ! { #[doc = " Set the URI of the request with separate scheme, authority, and"] #[doc = " path/query strings."] #[doc = ""] #[doc = " Each of `scheme`, `authority`, and `path_and_query` should either be"] #[doc = " null, to skip providing a component, or point to a UTF-8 encoded"] #[doc = " string. If any string pointer argument is non-null, its corresponding"] #[doc = " `len` parameter must be set to the string's length."] fn hyper_request_set_uri_parts (req : * mut hyper_request , scheme : * const u8 , scheme_len : size_t , authority : * const u8 , authority_len : size_t , path_and_query : * const u8 , path_and_query_len : size_t) -> hyper_code { let mut builder = Uri :: builder () ; if ! scheme . is_null () { let scheme_bytes = unsafe { std :: slice :: from_raw_parts (scheme , scheme_len as usize)}
; builder = builder . scheme (scheme_bytes) ;}
if ! authority . is_null () { let authority_bytes = unsafe { std :: slice :: from_raw_parts (authority , authority_len as usize)}
; builder = builder . authority (authority_bytes) ;}
if ! path_and_query . is_null () { let path_and_query_bytes = unsafe { std :: slice :: from_raw_parts (path_and_query , path_and_query_len as usize)}
; builder = builder . path_and_query (path_and_query_bytes) ;}
match builder . build () { Ok (u) => { * unsafe { & mut * req}
. 0 . uri_mut () = u ; hyper_code :: HYPERE_OK}
, Err (_) => { hyper_code :: HYPERE_INVALID_ARG}
}}
}