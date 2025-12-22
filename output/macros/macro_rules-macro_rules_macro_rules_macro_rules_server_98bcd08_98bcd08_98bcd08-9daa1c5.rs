macro_rules ! reply { ($ me : expr , $ res : expr , $ eos : expr) => { { match $ me . reply . send_response ($ res , $ eos) { Ok (tx) => tx , Err (e) => { debug ! ("send response error: {}" , e) ; $ me . reply . send_reset (Reason :: INTERNAL_ERROR) ; return Poll :: Ready (Err (crate :: Error :: new_h2 (e))) ;}
}}
} ; }