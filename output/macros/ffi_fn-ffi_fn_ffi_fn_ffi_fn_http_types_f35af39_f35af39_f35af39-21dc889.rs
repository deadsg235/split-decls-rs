ffi_fn ! { #[doc = " Iterates the headers passing each name and value pair to the callback."] #[doc = ""] #[doc = " The `userdata` pointer is also passed to the callback."] #[doc = ""] #[doc = " The callback should return `HYPER_ITER_CONTINUE` to keep iterating, or"] #[doc = " `HYPER_ITER_BREAK` to stop."] fn hyper_headers_foreach (headers : * const hyper_headers , func : hyper_headers_foreach_callback , userdata : * mut c_void) { let headers = non_null ! (&* headers ?= ()) ; let mut ordered_iter = headers . orig_order . get_in_order () . peekable () ; if ordered_iter . peek () . is_some () { for (name , idx) in ordered_iter { let (name_ptr , name_len) = if let Some (orig_name) = headers . orig_casing . get_all (name) . nth (* idx) { (orig_name . as_ref () . as_ptr () , orig_name . as_ref () . len ())}
else { (name . as_str () . as_bytes () . as_ptr () , name . as_str () . as_bytes () . len () ,)}
; let val_ptr ; let val_len ; if let Some (value) = headers . headers . get_all (name) . iter () . nth (* idx) { val_ptr = value . as_bytes () . as_ptr () ; val_len = value . as_bytes () . len () ;}
else { return ;}
if HYPER_ITER_CONTINUE != func (userdata , name_ptr , name_len , val_ptr , val_len) { return ;}
}}
else { for name in headers . headers . keys () { let mut names = headers . orig_casing . get_all (name) ; for value in headers . headers . get_all (name) { let (name_ptr , name_len) = if let Some (orig_name) = names . next () { (orig_name . as_ref () . as_ptr () , orig_name . as_ref () . len ())}
else { (name . as_str () . as_bytes () . as_ptr () , name . as_str () . as_bytes () . len () ,)}
; let val_ptr = value . as_bytes () . as_ptr () ; let val_len = value . as_bytes () . len () ; if HYPER_ITER_CONTINUE != func (userdata , name_ptr , name_len , val_ptr , val_len) { return ;}
}}
}}
}