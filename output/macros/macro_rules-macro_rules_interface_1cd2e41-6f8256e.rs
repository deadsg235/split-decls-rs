macro_rules ! error { ($ reason : expr) => { #[allow (rustc :: untranslatable_diagnostic)] #[allow (rustc :: diagnostic_outside_of_impl)] { let mut diag = dcx . struct_fatal (format ! ("invalid `--check-cfg` argument: `{s}`")) ; diag . note ($ reason) ; diag . note (VISIT) ; diag . emit ()}
} ; (in $ arg : expr , $ reason : expr) => { #[allow (rustc :: untranslatable_diagnostic)] #[allow (rustc :: diagnostic_outside_of_impl)] { let mut diag = dcx . struct_fatal (format ! ("invalid `--check-cfg` argument: `{s}`")) ; let pparg = rustc_ast_pretty :: pprust :: meta_list_item_to_string ($ arg) ; if let Some (lit) = $ arg . lit () { let (lit_kind_article , lit_kind_descr) = { let lit_kind = lit . as_token_lit () . kind ; (lit_kind . article () , lit_kind . descr ())}
; diag . note (format ! ("`{pparg}` is {lit_kind_article} {lit_kind_descr} literal")) ;}
else { diag . note (format ! ("`{pparg}` is invalid")) ;}
diag . note ($ reason) ; diag . note (VISIT) ; diag . emit ()}
} ; }