macro_rules ! print_tup { (num_should_render $ ($ ts : ident) *) => { 0 $ (+ $ ts . should_render () as usize) *}
; () => {}
; ($ t : ident $ ($ ts : ident) *) => { #[allow (non_snake_case , unused)] impl <$ t : PrintAttribute , $ ($ ts : PrintAttribute) ,*> PrintAttribute for ($ t , $ ($ ts) ,*) { fn should_render (& self) -> bool { let ($ t , $ ($ ts) ,*) = self ; print_tup ! (num_should_render $ t $ ($ ts) *) != 0}
fn print_attribute (& self , p : & mut Printer) { let ($ t , $ ($ ts) ,*) = self ; let parens = print_tup ! (num_should_render $ t $ ($ ts) *) > 1 ; if parens { p . popen () ;}
let mut printed_anything = $ t . should_render () ; $ t . print_attribute (p) ; $ (if $ ts . should_render () { if printed_anything { p . word_space (",") ;}
printed_anything = true ;}
$ ts . print_attribute (p) ;) * if parens { p . pclose () ;}
}}
print_tup ! ($ ($ ts) *) ;}
; }