macro_rules ! __internal_req_res_prop { (method : $ prop_val : expr) => { $ prop_val}
; (status : $ prop_val : expr) => { hyper :: StatusCode :: from_u16 ($ prop_val) . expect ("status code")}
; ($ prop_name : ident : $ prop_val : expr) => { From :: from ($ prop_val)}
; }