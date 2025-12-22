macro_rules ! CheckNoError { ($ err : ident) => { unsafe { assert ! ($ err . is_null () , "{}: {}" , phase , rstr ($ err)) ;}
} ; }