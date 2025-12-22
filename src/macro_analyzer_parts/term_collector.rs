use syn::{
    visit::Visit,
    Ident, LitInt, LitStr, LitBool, LitChar, LitByte, LitFloat,
    ExprCall, ExprMethodCall, ExprPath,
};
use crate::macro_analyzer_parts::terms::Term;

#[derive(Debug, Default)]
pub struct TermCollector {
    pub terms: Vec<Term>,
}

impl<'ast> Visit<'ast> for TermCollector {
    fn visit_lit_str(&mut self, i: &'ast LitStr) {
        self.terms.push(Term::StringLiteral(i.value()));
        syn::visit::visit_lit_str(self, i);
    }

    fn visit_lit_int(&mut self, i: &'ast LitInt) {
        self.terms.push(Term::NumericLiteral(i.to_string()));
        syn::visit::visit_lit_int(self, i);
    }

    fn visit_lit_bool(&mut self, i: &'ast LitBool) {
        self.terms.push(Term::BooleanLiteral(i.value()));
        syn::visit::visit_lit_bool(self, i);
    }

    fn visit_lit_char(&mut self, i: &'ast LitChar) {
        self.terms.push(Term::CharLiteral(i.value()));
        syn::visit::visit_lit_char(self, i);
    }

    fn visit_lit_byte(&mut self, i: &'ast LitByte) {
        self.terms.push(Term::ByteLiteral(i.value()));
        syn::visit::visit_lit_byte(self, i);
    }

    fn visit_lit_float(&mut self, i: &'ast LitFloat) {
        self.terms.push(Term::FloatLiteral(i.to_string()));
        syn::visit::visit_lit_float(self, i);
    }

    fn visit_ident(&mut self, i: &'ast Ident) {
        let name = i.to_string();
        // A more conservative filter for identifiers that are likely "terms"
        // Avoid common keywords, macro internal idents, and `syn` boilerplate
        if !name.starts_with("__") && // internal syn/proc-macro2 idents
           !name.starts_with("visit_") && // visitor methods
           ![
                "self", "super", "crate", "pub", "fn", "use", "mod", "impl", "for", "in", "let",
                "mut", "if", "else", "where", "macro_rules", "async", "await", "break", "continue",
                "enum", "extern", "false", "true", "loop", "match", "return", "static", "struct",
                "trait", "type", "union", "unsafe", "while", "const", "static", "move", "dyn",
                "ref", "box", "raw", "yield", "do", "typeof", "abstract", "become", "box", "final",
                "macro", "override", "priv", "typeof", "unsized", "virtual", "try", "union",
                "static_assert", "global", "unreachable", "default", "union",
                // Common type/trait idents that are usually not "terms" unless directly referenced
                "TokenStream", "Ident", "LitStr", "Punctuated", "Package", "CargoToml", "Dependency",
                "parse_macro_input", "quote", "ToTokens", "Result", "HashMap", "Path", "fs", "std",
                "String", "u64", "bool", "char", "u8", "Clone", "PartialEq", "Eq", "Hash", "Debug",
                "Default", "Vec", "ItemFn", "ExprCall", "ExprMethodCall", "ExprPath",
                "Token", "syn", "proc_macro", "proc_macro2", "cargo_toml_generator_types", "parsers",
                "DepTableInput", "WrappedCargoToml", "serde_json", "to_string", "from_str",
                "expect", "default", "to_string", "value", "span", "first", "last", "name",
                "table_content", "members", "version", "edition", "authors", "description",
                "homepage", "include", "keywords", "license", "publish", "repository", "rust_version",
                "name_lit", "version_lit", "path_lit", "name_ident", "_comma_token", "serialized",
                "cargo_toml_json",
            ].contains(&name.as_str()) &&
           !name.ends_with("_impl") // function implementation names
        {
            self.terms.push(Term::Identifier(name));
        }
        syn::visit::visit_ident(self, i);
    }

    fn visit_expr_call(&mut self, i: &'ast ExprCall) {
        // Extract the function name from a direct call
        if let syn::Expr::Path(ExprPath { path, .. }) = &*i.func {
            if let Some(segment) = path.segments.last() {
                self.terms.push(Term::FunctionCall(segment.ident.to_string()));
            }
        }
        syn::visit::visit_expr_call(self, i);
    }

    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        // Extract the method name
        self.terms.push(Term::FunctionCall(i.method.to_string()));
        syn::visit::visit_expr_method_call(self, i);
    }
}
