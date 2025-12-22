macro_rules ! cstrp { ($ s : expr) => { { static CSTR : & CStr = unsafe { CStr :: from_bytes_with_nul_unchecked (concat ! ($ s , "\0") . as_bytes ())}
; CSTR . as_ptr ()}
} ; }