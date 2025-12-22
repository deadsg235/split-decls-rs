#[doc = " Combines multiple lints passes into a single lint pass, at compile time,"] #[doc = " for maximum speed. Each `check_foo` method in `$methods` within this pass"] #[doc = " simply calls `check_foo` once per `$pass`. Compare with"] #[doc = " `RuntimeCombinedLateLintPass`, which is similar, but combines lint passes at"] #[doc = " runtime."] #[macro_export] macro_rules ! declare_combined_late_lint_pass { ([$ v : vis $ name : ident , [$ ($ pass : ident : $ constructor : expr ,) *]] , $ methods : tt) => (#[allow (non_snake_case)] $ v struct $ name { $ ($ pass : $ pass ,) *}
impl $ name { $ v fn new () -> Self { Self { $ ($ pass : $ constructor ,) *}
} $ v fn get_lints () -> $ crate :: LintVec { let mut lints = Vec :: new () ; $ (lints . extend_from_slice (&$ pass :: lint_vec ()) ;) * lints}
} impl <'tcx > $ crate :: LateLintPass <'tcx > for $ name { $ crate :: expand_combined_late_lint_pass_methods ! ([$ ($ pass) ,*] , $ methods) ;}
#[allow (rustc :: lint_pass_impl_without_macro)] impl $ crate :: LintPass for $ name { fn name (& self) -> &'static str { stringify ! ($ name)}
fn get_lints (& self) -> LintVec { $ name :: get_lints ()}
}) }