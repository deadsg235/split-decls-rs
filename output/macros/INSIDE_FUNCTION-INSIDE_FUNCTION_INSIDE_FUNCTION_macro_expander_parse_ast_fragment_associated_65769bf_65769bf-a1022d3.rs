INSIDE_FUNCTION ! { fn parse_ast_fragment (& mut self , toks : TokenStream , kind : AstFragmentKind , path : & ast :: Path , span : Span ,) -> AstFragment { let mut parser = self . cx . new_parser_from_tts (toks) ; match parse_ast_fragment (& mut parser , kind) { Ok (fragment) => { ensure_complete_parse (& parser , path , kind . name () , span) ; fragment}
Err (mut err) => { if err . span . is_dummy () { err . span (span) ;}
annotate_err_with_kind (& mut err , kind , span) ; let guar = err . emit () ; self . cx . macro_error_and_trace_macros_diag () ; kind . dummy (span , guar)}
}}
}