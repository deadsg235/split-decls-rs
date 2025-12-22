macro_rules ! __internal_map_prop { (headers : $ map : tt) => { { #[allow (unused_mut)] { let mut headers = HeaderMap :: new () ; __internal_headers_map ! (headers , $ map) ; headers}
}}
; ($ name : tt : $ val : tt) => { { __internal_req_res_prop ! ($ name : $ val)}
} ; }