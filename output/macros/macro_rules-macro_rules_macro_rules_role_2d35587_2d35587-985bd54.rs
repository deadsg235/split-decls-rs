macro_rules ! header_name { ($ bytes : expr) => { { match HeaderName :: from_bytes ($ bytes) { Ok (name) => name , Err (e) => maybe_panic ! (e) ,}
}}
; }