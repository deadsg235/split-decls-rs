macro_rules ! ffi_try_impl { ($ ($ function : ident) ::* ($ ($ arg : expr ,) *)) => { { let mut err : * mut :: libc :: c_char = :: std :: ptr :: null_mut () ; let result = $ ($ function) ::* ($ ($ arg ,) * & mut err) ; if ! err . is_null () { return Err (Error :: new ($ crate :: ffi_util :: error_message (err))) ;}
result}
} ; }